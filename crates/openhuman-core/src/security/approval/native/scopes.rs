//! The subjects the host registers with the module's approval engine.
//!
//! The module keeps one approval posture per subject (user, workspace, agent)
//! and refuses a call under an unregistered one. The host serves an unbounded
//! set of subjects (SaaS profiles, embedded agents, the posture variants of
//! one agent), so each is registered when first seen with
//! `RegisterApprovalScope2` and refreshed with `ReplaceApprovalScope2` when the
//! user's configuration changed: the "always allow" tool list, per-flow trust
//! and the auto-approve setting. The module only converts a required approval
//! into an audited automatic one as far as those say.

use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::Instant;

use tinysecurity_bus::{
    names, AccessTier, ApprovalGrants2, ApprovalScope2, ApprovalScopeReceipt2, FlowToolGrant2,
    RegisterApprovalScopeRequest2, SubjectScope, MAX_FLOW_GRANTS2, MAX_TOOL_GRANTS2,
};

use super::super::engine::Posture;
use super::rows::{digest_id, Rows};
use super::state::{handle_key, host};
use crate::config::Config;

/// The immutable partial-policy reference every approval row binds to. It
/// never changes with the posture, so a refreshed allowlist does not strand
/// the requests already pending.
const POLICY_REF: &str = "openhuman/approval/v1";

/// The key a subject is registered under.
pub(super) fn subject_key(subject: &SubjectScope) -> String {
    format!("{}\u{1f}{}\u{1f}{}", subject.user, subject.workspace, subject.agent)
}

/// One registered subject.
pub(super) struct ScopeEntry {
    pub(super) subject: SubjectScope,
    pub(super) scope_id: String,
    pub(super) stream_id: String,
    pub(super) rows: Rows,
    handle: OnceLock<SubjectScope>,
    /// Held across every module call that appends an audit event for this
    /// subject, so the tail the host reserves is the tail the module extends.
    pub(super) stream: tokio::sync::Mutex<()>,
    /// What the module currently holds for the subject.
    registered: tokio::sync::Mutex<Option<String>>,
    swept: Mutex<Option<Instant>>,
}

impl std::fmt::Debug for ScopeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScopeEntry").finish_non_exhaustive()
    }
}

impl ScopeEntry {
    fn new(subject: SubjectScope, stream_id: String, rows: Rows) -> Self {
        let scope_id = digest_id(&["scope", &subject.user, &subject.workspace, &subject.agent]);
        Self {
            subject,
            scope_id,
            stream_id,
            rows,
            handle: OnceLock::new(),
            stream: tokio::sync::Mutex::new(()),
            registered: tokio::sync::Mutex::new(None),
            swept: Mutex::new(None),
        }
    }

    /// The module's handle for the subject, known once it is registered.
    pub(super) fn handle(&self) -> Option<&SubjectScope> {
        self.handle.get()
    }

    /// Whether a stale-request sweep is due, and mark it done.
    pub(super) fn sweep_due(&self, every: std::time::Duration) -> bool {
        let mut swept = self.swept.lock().unwrap_or_else(PoisonError::into_inner);
        if swept.is_some_and(|at| at.elapsed() < every) {
            return false;
        }
        *swept = Some(Instant::now());
        true
    }

    /// Make the module hold `posture` for this subject.
    async fn register(
        &self,
        proxy: &tinybus::Proxy,
        posture: &Posture,
        generation: u64,
    ) -> Result<(), String> {
        let grants = grants(posture);
        let fingerprint = serde_json::to_string(&(
            posture.require_approval,
            posture.auto_approve_all,
            &grants.tools,
            grants
                .flows
                .iter()
                .map(|g| (&g.flow_id, &g.tool))
                .collect::<Vec<_>>(),
        ))
        .map_err(|error| error.to_string())?;
        let mut registered = self.registered.lock().await;
        if registered.as_deref() == Some(fingerprint.as_str()) {
            return Ok(());
        }
        let first = registered.is_none();
        let request = RegisterApprovalScopeRequest2 {
            scope: ApprovalScope2 {
                scope_id: self.scope_id.clone(),
                subject: self.subject.clone(),
                policy_ref: POLICY_REF.into(),
                generation,
                ceiling: AccessTier::Full,
                require_approval: posture.require_approval,
                auto_approve_all: posture.auto_approve_all,
            },
            grants,
            audit_stream_id: self.stream_id.clone(),
        };
        let member = if first {
            names::methods::REGISTER_APPROVAL_SCOPE2
        } else {
            names::methods::REPLACE_APPROVAL_SCOPE2
        };
        let receipt: ApprovalScopeReceipt2 = proxy
            .call_confidential(member, (request,))
            .await
            .map_err(|error| {
                tracing::warn!(
                    member,
                    error = %error,
                    "[approval::native] the module refused the approval scope"
                );
                format!("TinySecurity {member} failed")
            })?;
        if receipt.scope_id != self.scope_id {
            return Err("TinySecurity returned a receipt for another approval scope".into());
        }
        if self.handle.get().is_none() {
            let handle = receipt.scope_handle.clone();
            // Callbacks look the subject up by this handle, so it is known before
            // the first call under the scope can reach them.
            host()
                .scopes
                .lock()
                .await
                .by_handle
                .insert(handle_key(&handle), Arc::new_cyclic_free(self));
            let _ = self.handle.set(handle);
        }
        tracing::debug!(
            scope = %self.scope_id,
            outcome = ?receipt.outcome,
            live_scopes = receipt.live_scopes,
            "[approval::native] approval scope current"
        );
        *registered = Some(fingerprint);
        Ok(())
    }
}

/// The user's configured grants, narrowed to what the module accepts: a name it
/// refuses (a human-only or native tool) can never be granted, so it is dropped
/// here rather than failing the whole registration.
fn grants(posture: &Posture) -> ApprovalGrants2 {
    let accepted = |tools: &[String], flows: &[FlowToolGrant2]| {
        ApprovalGrants2 {
            tools: tools.to_vec(),
            flows: flows.to_vec(),
        }
        .is_valid()
    };
    let mut tools: Vec<String> = posture
        .tool_grants
        .iter()
        .filter(|tool| accepted(std::slice::from_ref(tool), &[]))
        .cloned()
        .collect();
    tools.sort();
    tools.dedup();
    tools.truncate(MAX_TOOL_GRANTS2);
    let mut flows: Vec<FlowToolGrant2> = posture
        .flow_grants
        .iter()
        .map(|(flow_id, tool)| FlowToolGrant2 {
            flow_id: flow_id.clone(),
            tool: tool.clone(),
        })
        .filter(|grant| accepted(&[], std::slice::from_ref(grant)))
        .collect();
    flows.sort_by(|a, b| (&a.flow_id, &a.tool).cmp(&(&b.flow_id, &b.tool)));
    flows.dedup_by(|a, b| a.flow_id == b.flow_id && a.tool == b.tool);
    flows.truncate(MAX_FLOW_GRANTS2);
    ApprovalGrants2 { tools, flows }
}

/// The registered subject for `subject`, registering it or refreshing its
/// posture first.
pub(super) async fn ensure(
    config: &Config,
    proxy: &tinybus::Proxy,
    subject: SubjectScope,
    posture: &Posture,
    generation: u64,
) -> Result<Arc<ScopeEntry>, String> {
    let key = subject_key(&subject);
    let entry = {
        let mut scopes = host().scopes.lock().await;
        match scopes.by_subject.get(&key) {
            Some(found) => Arc::clone(found),
            None => {
                let rows = Rows::current(config).map_err(|error| error.to_string())?;
                let stream_id = rows.stream_for(&key).await?;
                let entry = Arc::new(ScopeEntry::new(subject, stream_id, rows));
                scopes.by_subject.insert(key, Arc::clone(&entry));
                entry
            }
        }
    };
    entry.register(proxy, posture, generation).await?;
    Ok(entry)
}

/// The registered subject a module callback names by its handle.
pub(super) async fn by_handle(handle: &SubjectScope) -> Option<Arc<ScopeEntry>> {
    host()
        .scopes
        .lock()
        .await
        .by_handle
        .get(&handle_key(handle))
        .cloned()
}

#[cfg(test)]
#[path = "scopes_tests.rs"]
mod tests;
