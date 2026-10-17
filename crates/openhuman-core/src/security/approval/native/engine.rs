//! `NativeEngine`: the approval lifecycle, asked of the TinySecurity module.

use std::time::Duration;

use async_trait::async_trait;
use tinybus::Proxy;
use tinysecurity_bus::{
    is_host_tool_name2, names, AccessTier, ApprovalActionRequest2, ApprovalChoice2,
    ApprovalLifecycle2, ApprovalRecordRequest2, ApprovalRow2, AuditCorrelation2, CallerContext,
    Check, DecideRequest2, EvaluateApprovalRequest2, EvaluateApprovalResponse2, EvaluateRequest,
    ExecutionOutcome as ModuleOutcome, FlowTrustList2, FlowTrustOutcome2, FlowTrustReceipt2,
    GrantFlowToolRequest2, ListFlowTrustRequest2, Origin, PolicyInfo, RecordExecutionRequest2,
    RevokeFlowToolRequest2, SubjectScope, ToolCall, Verdict,
};

use super::super::engine::{
    ApprovalEngine, CallOrigin, CheckInput, Checked, Choice, Decided, EngineError, Sealed, Variant,
};
use super::super::types::{ApprovalAuditEntry, ExecutionOutcome};
use super::scopes::{self, ScopeEntry};
use super::state::{host, CallEntry, RequestEntry, Ticket};
use crate::config::Config;
use crate::modules::security::policy_translation;

const LOG: &str = "[approval::native]";

/// The deciding actor the module records for a decision made through the host's
/// own authenticated approval surface.
const ACTOR: &str = "operator";

/// How long one module call may take: past the longest callback deadline the
/// module itself enforces (store, audit, prompt, judge), so a slow callback
/// surfaces as the module's own refusal rather than a host timeout.
const CALL_DEADLINE: Duration = Duration::from_secs(25);

/// How often a subject's stale pending requests are expired.
const SWEEP_EVERY: Duration = Duration::from_secs(30);

/// How long past its window a request still held in memory is left to its own
/// timer before the sweep expires it.
const SWEEP_GRACE_MS: u64 = 30_000;

/// Stale requests expired per sweep.
const SWEEP_BATCH: usize = 25;

/// Ends a call's wait for its prompt however the check returns.
struct CallGuard(String);

impl Drop for CallGuard {
    fn drop(&mut self) {
        host().end_call(&self.0);
    }
}

/// Where the engine gets its bus proxy.
enum Source {
    /// The attested, digest-pinned module the registry loads.
    Production,
    /// A proxy a test supplies for a module it loaded itself.
    Fixed(Proxy),
}

/// The approval engine over the module's bus contract.
pub(crate) struct NativeEngine {
    config: Config,
    source: Source,
}

impl std::fmt::Debug for NativeEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeEngine").finish_non_exhaustive()
    }
}

impl Sealed for NativeEngine {}

fn unavailable(message: impl Into<String>) -> EngineError {
    EngineError::Unavailable(message.into())
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn opaque() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

fn module_origin(origin: CallOrigin) -> Origin {
    match origin {
        CallOrigin::Operator => Origin::Operator,
        CallOrigin::Copilot => Origin::Copilot,
        CallOrigin::SubAgent => Origin::SubAgent,
        CallOrigin::Cron => Origin::Cron,
        CallOrigin::TrustedAutomation => Origin::TrustedAutomation,
        CallOrigin::LocalOperator => Origin::LocalOperator,
        CallOrigin::Unlabelled => Origin::Unlabelled,
    }
}

fn module_outcome(outcome: ExecutionOutcome) -> ModuleOutcome {
    match outcome {
        ExecutionOutcome::Success => ModuleOutcome::Succeeded,
        ExecutionOutcome::Failure => ModuleOutcome::Failed,
        ExecutionOutcome::Aborted => ModuleOutcome::Cancelled,
    }
}

impl NativeEngine {
    /// The engine over the registry's module, for `config`.
    pub(crate) fn production(config: Config) -> Self {
        Self {
            config,
            source: Source::Production,
        }
    }

    /// The engine over a module a test loaded itself.
    #[cfg(test)]
    pub(crate) fn on(config: Config, proxy: Proxy) -> Self {
        Self {
            config,
            source: Source::Fixed(proxy),
        }
    }

    async fn proxy(&self) -> Result<Proxy, EngineError> {
        match &self.source {
            Source::Fixed(proxy) => Ok(proxy.clone()),
            Source::Production => crate::modules::security::approval_proxy(&self.config)
                .await
                .map(|proxy| proxy.with_timeout(CALL_DEADLINE))
                .map_err(EngineError::Unavailable),
        }
    }

    /// The subject a call runs under: the tenant, the workspace and the agent,
    /// with the variant that selects its posture.
    fn subject(
        &self,
        tenant: &crate::core::runtime::Tenant,
        variant: Variant,
    ) -> Result<SubjectScope, EngineError> {
        let mut subject = policy_translation::subject_scope(
            tenant,
            &self.config.workspace_dir,
            crate::core::runtime::is_saas(),
        )
        .map_err(EngineError::Unavailable)?;
        subject.agent.push_str(variant.suffix());
        Ok(subject)
    }

    /// The registered scope a flow's durable trust lives under: the tenant's
    /// strict-flow subject, registered with the restrictive posture if it is new
    /// (the next gated call refreshes it to the user's configuration).
    async fn flow_scope(
        &self,
        proxy: &Proxy,
        tenant: &crate::core::runtime::Tenant,
    ) -> Result<std::sync::Arc<ScopeEntry>, EngineError> {
        let subject = self.subject(tenant, Variant::FlowStrict)?;
        let generation = Self::generation(proxy).await?;
        scopes::ensure(
            &self.config,
            proxy,
            subject,
            &super::super::engine::Posture {
                require_approval: true,
                ..Default::default()
            },
            generation,
        )
        .await
        .map_err(EngineError::Unavailable)
    }

    /// The module's current generation, which every correlation carries.
    async fn generation(proxy: &Proxy) -> Result<u64, EngineError> {
        let info: PolicyInfo = proxy
            .call(names::methods::POLICY_INFO, ())
            .await
            .map_err(|_| unavailable("TinySecurity PolicyInfo failed"))?;
        Ok(info.generation)
    }

    /// The correlation of the next audit event of a subject. The caller holds the
    /// subject's stream lock, so the tail read here is the tail the module extends.
    async fn correlation(
        scope: &ScopeEntry,
        generation: u64,
    ) -> Result<AuditCorrelation2, EngineError> {
        let expected_tail = scope
            .rows
            .tail(&scope.stream_id)
            .await
            .map_err(EngineError::Unavailable)?;
        Ok(AuditCorrelation2 {
            event_id: opaque(),
            expected_tail,
            generation,
        })
    }

    fn action(
        entry: &RequestEntry,
        request_id: &str,
        audit: AuditCorrelation2,
    ) -> ApprovalActionRequest2 {
        ApprovalActionRequest2 {
            scope_id: entry.scope.scope_id.clone(),
            request_id: request_id.to_owned(),
            request: entry.request.clone(),
            audit,
        }
    }

    /// What a row became after a call failed, so the caller can tell a lost race
    /// from a failure.
    async fn settled(entry: &RequestEntry, request_id: &str) -> bool {
        match entry.scope.rows.row(request_id).await {
            Ok(Some(stored)) => stored.row.state != ApprovalLifecycle2::Pending,
            _ => false,
        }
    }

    /// Expire this subject's requests whose window elapsed and that nobody will
    /// answer. A failure is logged: the sweep only keeps the module's view small.
    async fn sweep(&self, proxy: &Proxy, scope: &std::sync::Arc<ScopeEntry>, generation: u64) {
        if !scope.sweep_due(SWEEP_EVERY) {
            return;
        }
        let stale = match scope.rows.stale(now_ms(), SWEEP_BATCH).await {
            Ok(stale) => stale,
            Err(error) => {
                tracing::debug!(error = %error, "{LOG} stale sweep could not list rows");
                return;
            }
        };
        for stored in stale {
            let request_id = stored.row.request_id;
            // A request still parked in this process expires through its own
            // timer, which also settles the call waiting on it. One that outlived
            // its window by the grace had its park abandoned (a caller bound),
            // so nothing else will ever close it.
            if host().request(&request_id).is_some()
                && now_ms() < stored.row.expires_at_ms.saturating_add(SWEEP_GRACE_MS)
            {
                continue;
            }
            let _guard = scope.stream.lock().await;
            let Ok(audit) = Self::correlation(scope, generation).await else {
                return;
            };
            let outcome: Result<ApprovalRow2, _> = proxy
                .call_confidential(
                    names::methods::EXPIRE_APPROVAL2,
                    (ApprovalRecordRequest2 {
                        scope_id: scope.scope_id.clone(),
                        request_id: request_id.clone(),
                        audit,
                    },),
                )
                .await;
            match outcome {
                Ok(_) => tracing::debug!(request_id = %request_id, "{LOG} swept a stale request"),
                Err(error) => tracing::debug!(
                    request_id = %request_id,
                    error = %error,
                    "{LOG} stale request left for the next sweep"
                ),
            }
        }
    }
}

#[async_trait]
impl ApprovalEngine for NativeEngine {
    async fn check(&self, input: CheckInput) -> Result<Checked, EngineError> {
        if !is_host_tool_name2(&input.tool) {
            tracing::warn!(tool = %input.tool, "{LOG} the module cannot gate this tool name");
            return Err(unavailable(format!(
                "'{}' cannot be gated by the approval engine",
                input.tool
            )));
        }
        let proxy = self.proxy().await?;
        let subject = self.subject(&input.tenant, input.variant)?;
        let generation = Self::generation(&proxy).await?;
        let scope = scopes::ensure(
            &self.config,
            &proxy,
            subject.clone(),
            &input.posture,
            generation,
        )
        .await
        .map_err(EngineError::Unavailable)?;
        self.sweep(&proxy, &scope, generation).await;

        let arguments_json =
            serde_json::to_string(&input.args).map_err(|e| unavailable(e.to_string()))?;
        let request = EvaluateRequest {
            caller: CallerContext {
                agent: subject.agent,
                thread: input.thread,
                flow: input.flow,
                call_id: input.call_id.clone(),
                origin: module_origin(input.origin),
                tier: AccessTier::Full,
            },
            call: ToolCall {
                tool: input.tool,
                arguments_json,
                checks: vec![Check::HostTool {}],
            },
        };
        if !host().begin_call(
            &input.call_id,
            CallEntry {
                prompt: input.prompt,
                scope: std::sync::Arc::clone(&scope),
                request: request.clone(),
            },
        ) {
            return Err(unavailable("the approval correlation is already in use"));
        }
        let _call = CallGuard(input.call_id.clone());
        let response = {
            let _guard = scope.stream.lock().await;
            let generation = Self::generation(&proxy).await?;
            let audit = Self::correlation(&scope, generation).await?;
            let ttl_ms = input
                .ttl
                .map(|ttl| u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX).max(1));
            let evaluated: Result<EvaluateApprovalResponse2, _> = proxy
                .call_confidential(
                    names::methods::EVALUATE_APPROVAL2,
                    (EvaluateApprovalRequest2 {
                        scope_id: scope.scope_id.clone(),
                        request: request.clone(),
                        ttl_ms,
                        summary: input.summary,
                        audit,
                    },),
                )
                .await;
            evaluated.map_err(|error| {
                tracing::warn!(error = %error, "{LOG} the module refused to evaluate the call");
                // The module's own `Denied` is an answer; anything else is a failure.
                if matches!(
                    &error,
                    tinybus::Error::MethodFailed { message, .. } if message == "approval denied"
                ) {
                    return EngineError::Refused;
                }
                unavailable(format!("TinySecurity EvaluateApproval2 failed: {error}"))
            })?
        };
        let record = response.record;
        match response.decision.verdict {
            Verdict::Allow if record.state == ApprovalLifecycle2::Approved => {
                host().insert_request(&record.request_id, RequestEntry::new(scope, request, None));
                tracing::debug!(
                    request_id = %record.request_id,
                    attribution = record.attribution.as_deref().unwrap_or(""),
                    "{LOG} approved without a human"
                );
                Ok(Checked::Approved {
                    request_id: record.request_id,
                })
            }
            Verdict::RequireApproval { request_id }
                if request_id == record.request_id
                    && record.state == ApprovalLifecycle2::Pending =>
            {
                if host().request(&request_id).is_none() {
                    tracing::warn!(request_id = %request_id, "{LOG} pending without a delivered prompt");
                    return Err(unavailable("the approval prompt was not delivered"));
                }
                Ok(Checked::Pending {
                    request_id,
                    expires_at_ms: record.expires_at_ms,
                })
            }
            other => {
                tracing::debug!(verdict = ?other, "{LOG} the module did not approve the call");
                Err(unavailable("the approval engine refused the call"))
            }
        }
    }

    async fn decide(
        &self,
        request_id: &str,
        choice: Choice,
        label: &str,
    ) -> Result<Decided, EngineError> {
        let entry = host().request(request_id).ok_or(EngineError::NotPending)?;
        if entry.delivered.is_none() {
            // Approved without a human: there is nothing to decide.
            return Err(EngineError::NotPending);
        }
        let proxy = self.proxy().await?;
        let ticket = opaque();
        host().issue(
            ticket.clone(),
            Ticket::new(
                request_id.to_owned(),
                match choice {
                    Choice::Once => ApprovalChoice2::Once,
                    Choice::Deny => ApprovalChoice2::Deny,
                },
                ACTOR.to_owned(),
            ),
        );
        let decided = async {
            let _guard = entry.scope.stream.lock().await;
            let generation = Self::generation(&proxy).await?;
            let audit = Self::correlation(&entry.scope, generation).await?;
            proxy
                .call_confidential::<ApprovalRow2>(
                    names::methods::DECIDE2,
                    (DecideRequest2 {
                        scope_id: entry.scope.scope_id.clone(),
                        request_id: request_id.to_owned(),
                        decision_ticket: ticket.clone(),
                        audit,
                    },),
                )
                .await
                .map_err(|error| {
                    tracing::debug!(error = %error, "{LOG} the module refused the decision");
                    unavailable(format!("TinySecurity Decide2 failed: {error}"))
                })
        }
        .await;
        host().revoke(&ticket);
        match decided {
            Ok(row) => {
                if let Err(error) = entry.scope.rows.put_label(request_id, label).await {
                    tracing::debug!(error = %error, "{LOG} the decision label was not kept");
                }
                match row.state {
                    ApprovalLifecycle2::Approved => Ok(Decided::Approved),
                    ApprovalLifecycle2::Denied => Ok(Decided::Denied),
                    _ => Err(EngineError::NotPending),
                }
            }
            Err(error) => {
                if Self::settled(&entry, request_id).await {
                    Err(EngineError::NotPending)
                } else {
                    Err(error)
                }
            }
        }
    }

    async fn reserve(&self, request_id: &str) -> Result<(), EngineError> {
        let entry = host().request(request_id).ok_or(EngineError::NotPending)?;
        let proxy = self.proxy().await?;
        let _guard = entry.scope.stream.lock().await;
        let generation = Self::generation(&proxy).await?;
        let audit = Self::correlation(&entry.scope, generation).await?;
        let action = Self::action(&entry, request_id, audit);
        let row: ApprovalRow2 = proxy
            .call_confidential(names::methods::CONSUME_APPROVAL2, (action,))
            .await
            .map_err(|error| {
                tracing::warn!(error = %error, "{LOG} the module refused to reserve the call");
                unavailable(format!("TinySecurity ConsumeApproval2 failed: {error}"))
            })?;
        if row.state == ApprovalLifecycle2::Reserved {
            Ok(())
        } else {
            Err(unavailable("the approval was not reserved"))
        }
    }

    async fn record_execution(
        &self,
        request_id: &str,
        outcome: ExecutionOutcome,
    ) -> Result<(), EngineError> {
        let entry = host().request(request_id).ok_or(EngineError::NotPending)?;
        let proxy = self.proxy().await?;
        {
            let _guard = entry.scope.stream.lock().await;
            let generation = Self::generation(&proxy).await?;
            let audit = Self::correlation(&entry.scope, generation).await?;
            let _: ApprovalRow2 = proxy
                .call_confidential(
                    names::methods::RECORD_EXECUTION2,
                    (RecordExecutionRequest2 {
                        action: Self::action(&entry, request_id, audit),
                        outcome: module_outcome(outcome),
                    },),
                )
                .await
                .map_err(|error| {
                    tracing::warn!(error = %error, "{LOG} the module refused the execution record");
                    unavailable(format!("TinySecurity RecordExecution2 failed: {error}"))
                })?;
        }
        host().forget_request(request_id);
        Ok(())
    }

    async fn expire(&self, request_id: &str) -> Result<bool, EngineError> {
        let entry = host().request(request_id).ok_or(EngineError::NotPending)?;
        let proxy = self.proxy().await?;
        let expired = {
            let _guard = entry.scope.stream.lock().await;
            let generation = Self::generation(&proxy).await?;
            let audit = Self::correlation(&entry.scope, generation).await?;
            proxy
                .call_confidential::<ApprovalRow2>(
                    names::methods::EXPIRE_APPROVAL2,
                    (ApprovalRecordRequest2 {
                        scope_id: entry.scope.scope_id.clone(),
                        request_id: request_id.to_owned(),
                        audit,
                    },),
                )
                .await
        };
        match expired {
            Ok(row) => {
                host().forget_request(request_id);
                Ok(row.state == ApprovalLifecycle2::Expired)
            }
            Err(error) => {
                if Self::settled(&entry, request_id).await {
                    // Decided in the same instant: the caller honors the decision.
                    Ok(false)
                } else {
                    tracing::warn!(error = %error, "{LOG} the module refused to expire the request");
                    Err(unavailable(format!(
                        "TinySecurity ExpireApproval2 failed: {error}"
                    )))
                }
            }
        }
    }

    fn release(&self, request_id: &str) {
        host().forget_request(request_id);
    }

    async fn cancel(&self, request_id: &str) -> Result<(), EngineError> {
        let entry = host().request(request_id).ok_or(EngineError::NotPending)?;
        let proxy = self.proxy().await?;
        let cancelled = {
            let _guard = entry.scope.stream.lock().await;
            let generation = Self::generation(&proxy).await?;
            let audit = Self::correlation(&entry.scope, generation).await?;
            proxy
                .call_confidential::<ApprovalRow2>(
                    names::methods::CANCEL_APPROVAL2,
                    (ApprovalRecordRequest2 {
                        scope_id: entry.scope.scope_id.clone(),
                        request_id: request_id.to_owned(),
                        audit,
                    },),
                )
                .await
        };
        host().forget_request(request_id);
        cancelled.map(|_| ()).map_err(|error| {
            tracing::debug!(error = %error, "{LOG} the module did not cancel the request");
            unavailable(format!("TinySecurity CancelApproval2 failed: {error}"))
        })
    }

    async fn is_decided(&self, request_id: &str) -> Result<Option<bool>, EngineError> {
        super::view::row_decided(&self.config, request_id)
            .await
            .map_err(|error| unavailable(error.to_string()))
    }

    async fn history(&self, limit: usize) -> Result<Vec<ApprovalAuditEntry>, EngineError> {
        super::view::decided_history(&self.config, limit)
            .await
            .map_err(|error| unavailable(error.to_string()))
    }

    async fn grant_flow_trust(
        &self,
        tenant: &crate::core::runtime::Tenant,
        flow_id: &str,
        tool: &str,
    ) -> Result<(), EngineError> {
        let proxy = self.proxy().await?;
        let scope = self.flow_scope(&proxy, tenant).await?;
        let _guard = scope.stream.lock().await;
        let generation = Self::generation(&proxy).await?;
        let audit = Self::correlation(&scope, generation).await?;
        let receipt: FlowTrustReceipt2 = proxy
            .call_confidential(
                names::methods::GRANT_FLOW_TOOL2,
                (GrantFlowToolRequest2 {
                    scope_id: scope.scope_id.clone(),
                    flow_id: flow_id.to_owned(),
                    tool: tool.to_owned(),
                    audit,
                },),
            )
            .await
            .map_err(|error| {
                tracing::warn!(error = %error, "{LOG} the module refused the flow trust");
                unavailable(format!("TinySecurity GrantFlowTool2 failed: {error}"))
            })?;
        tracing::debug!(outcome = ?receipt.outcome, "{LOG} flow trust granted");
        Ok(())
    }

    async fn revoke_flow_trust(
        &self,
        tenant: &crate::core::runtime::Tenant,
        flow_id: &str,
        tool: &str,
    ) -> Result<bool, EngineError> {
        let proxy = self.proxy().await?;
        let scope = self.flow_scope(&proxy, tenant).await?;
        let _guard = scope.stream.lock().await;
        let generation = Self::generation(&proxy).await?;
        let audit = Self::correlation(&scope, generation).await?;
        let receipt: FlowTrustReceipt2 = proxy
            .call_confidential(
                names::methods::REVOKE_FLOW_TOOL2,
                (RevokeFlowToolRequest2 {
                    scope_id: scope.scope_id.clone(),
                    flow_id: flow_id.to_owned(),
                    tool: tool.to_owned(),
                    audit,
                },),
            )
            .await
            .map_err(|error| {
                tracing::warn!(error = %error, "{LOG} the module refused to revoke flow trust");
                unavailable(format!("TinySecurity RevokeFlowTool2 failed: {error}"))
            })?;
        Ok(receipt.outcome == FlowTrustOutcome2::Revoked)
    }

    async fn flow_trust(
        &self,
        tenant: &crate::core::runtime::Tenant,
        flow_id: &str,
    ) -> Result<Vec<String>, EngineError> {
        let proxy = self.proxy().await?;
        let scope = self.flow_scope(&proxy, tenant).await?;
        let listed: FlowTrustList2 = proxy
            .call_confidential(
                names::methods::LIST_FLOW_TRUST2,
                (ListFlowTrustRequest2 {
                    scope_id: scope.scope_id.clone(),
                    flow_id: Some(flow_id.to_owned()),
                },),
            )
            .await
            .map_err(|error| {
                unavailable(format!("TinySecurity ListFlowTrust2 failed: {error}"))
            })?;
        let mut tools: Vec<String> = listed.entries.into_iter().map(|e| e.tool).collect();
        tools.sort();
        tools.dedup();
        Ok(tools)
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
