//! The one security gate every agent path installs as its outermost tool
//! middleware (issue #7328, phase 6).
//!
//! With the `security-module` feature the gate is the harness
//! [`ToolPolicyGateMiddleware`] over [`TinySecurityPolicy`]: the module's
//! decision is mapped onto the harness decision (deny beats require-approval
//! beats allow, a module fault denies with no retry) and a required approval
//! settles through OpenHuman's approval resolver. Without the feature there is
//! no module, so the resolver gate alone remains.
//!
//! Turns (hosted roots included), sub-agents, flows and the live voice harness all
//! install it through [`install`], so no path carries its own copy of the wiring.
//! A session with a channel tool policy adds one more verdict source for the
//! channel's "ask a human first" tools ([`super::tinysecurity_channel`]).

use std::sync::Arc;

use tinyagents_harness::context::RunContext;
use tinyagents_harness::middleware::{PolicyDecision, ToolCallPolicy};
use tinyagents_harness::runtime::AgentHarness;
use tinyinference_llm::tool::ToolCall as TaToolCall;
use tinytools::Tool;

use super::approval::ApprovalSecurityMiddleware;
use crate::agent::tinyagents::host::OpenHumanRunContext;
use crate::agent::tinyagents::ToolPolicyEnforcement;

/// Folds independent host and module policies before the single resolver sees
/// an approval request. A denial from any source dominates an approval from
/// another source.
struct CombinedPolicy {
    policies: Vec<Arc<dyn ToolCallPolicy<OpenHumanRunContext>>>,
}

#[async_trait::async_trait]
impl ToolCallPolicy<OpenHumanRunContext> for CombinedPolicy {
    fn name(&self) -> &str {
        "openhuman_security_gate"
    }

    async fn check(
        &self,
        ctx: &RunContext<OpenHumanRunContext>,
        call: &TaToolCall,
    ) -> PolicyDecision {
        let mut decisions = Vec::with_capacity(self.policies.len());
        for policy in &self.policies {
            decisions.push(policy.check(ctx, call).await);
        }
        let mut combined = PolicyDecision::Allow;
        for decision in decisions {
            match (&combined, &decision) {
                (PolicyDecision::Deny { .. }, _) => {}
                (_, PolicyDecision::Deny { .. }) => combined = decision,
                (PolicyDecision::RequireApproval { .. }, _) => {}
                (_, PolicyDecision::RequireApproval { .. }) => combined = decision,
                _ => {}
            }
        }
        combined
    }
}

/// Feature-off policy: without TinySecurity, only inert reads may execute.
#[cfg(not(feature = "security-module"))]
struct MissingModulePolicy {
    effects: ApprovalSecurityMiddleware,
}

#[cfg(not(feature = "security-module"))]
#[async_trait::async_trait]
impl ToolCallPolicy<OpenHumanRunContext> for MissingModulePolicy {
    fn name(&self) -> &str {
        "tinysecurity_unavailable"
    }

    async fn check(
        &self,
        _ctx: &RunContext<OpenHumanRunContext>,
        call: &TaToolCall,
    ) -> PolicyDecision {
        let Some(tool) = self.effects.resolve_tool(&call.name) else {
            return PolicyDecision::deny("security module unavailable for unknown tool");
        };
        let declaration = tool.policy();
        let effects = declaration.side_effects;
        let explicitly_inert = declaration.classified
            && effects.read_only
            && !effects.writes_files
            && !effects.network
            && !effects.installs_dependencies
            && !effects.destructive
            && !effects.external_service
            && !effects.payment;
        if !explicitly_inert
            || tool.permission_level_with_args(&call.arguments)
                > tinytools::PermissionLevel::ReadOnly
            || tool.external_effect_with_args(&call.arguments)
        {
            PolicyDecision::deny("security module unavailable for acting tool")
        } else {
            PolicyDecision::Allow
        }
    }
}

/// The gate over `policy`, settling a required approval through `resolver`.
/// Denials render in the structured `Blocked / Reason / Workaround` shape the
/// rest of the host's policy boundaries use.
pub(crate) fn gate(
    policy: Arc<dyn tinyagents_harness::middleware::ToolCallPolicy<OpenHumanRunContext>>,
    resolver: Arc<dyn tinyagents_harness::middleware::ApprovalResolver<OpenHumanRunContext>>,
) -> tinyagents_harness::middleware::ToolPolicyGateMiddleware<OpenHumanRunContext> {
    use crate::agent::tinyagents::policy_denial::PolicyDenial;
    use tinyagents_harness::middleware::{PolicyDecision, ToolPolicyGateMiddleware};

    ToolPolicyGateMiddleware::new(policy)
        .with_approval_resolver(resolver)
        .with_denial_renderer(Arc::new(|call, policy, decision| {
            let denial = match decision {
                PolicyDecision::RequireApproval { reason } => PolicyDenial::ApprovalRequired {
                    tool: &call.name,
                    policy,
                    reason,
                },
                // A refusal that already carries the machine-recognisable marker
                // keeps it on the `Blocked` line, where classification and the
                // loop breaker look for it.
                PolicyDecision::Deny { reason }
                    if reason.contains(tinytools::status::POLICY_BLOCKED_MARKER) =>
                {
                    PolicyDenial::SecurityPolicyBlocked {
                        tool: &call.name,
                        raw_reason: reason,
                    }
                }
                PolicyDecision::Deny { reason } => PolicyDenial::PolicyDenied {
                    tool: &call.name,
                    policy,
                    reason,
                },
                PolicyDecision::Allow => PolicyDenial::PolicyDenied {
                    tool: &call.name,
                    policy,
                    reason: "",
                },
            };
            denial.render()
        }))
}

/// The security gate as a tool middleware over `tool_sets`: the one construction
/// every path that runs a tool shares, so a harness turn ([`install`]) and a direct
/// run (`tools::native_ops::execute_tool`, the JS, RPC and workflow tool calls)
/// are judged by the same policy and settle approvals through the same resolver.
///
/// `channel` is the session's channel tool policy, for a session that has one.
pub(crate) fn tool_middleware(
    tool_sets: Vec<Arc<Vec<Box<dyn Tool>>>>,
    channel: Option<tinytools::ToolPolicySession>,
    enforcement: Option<ToolPolicyEnforcement>,
) -> Arc<dyn tinyagents_harness::middleware::ToolMiddleware<(), OpenHumanRunContext>> {
    #[cfg(feature = "security-module")]
    {
        use super::tinysecurity::{TinySecurityPolicy, VerdictSource};
        use super::tinysecurity_channel::ChannelApprovalVerdicts;
        use super::tinysecurity_module::ModuleVerdicts;

        let mut sources: Vec<Arc<dyn VerdictSource>> =
            vec![Arc::new(ModuleVerdicts::new(tool_sets.clone()))];
        if let Some(session) = channel {
            sources.push(Arc::new(ChannelApprovalVerdicts::new(session)));
        }
        let security_policy: Arc<dyn ToolCallPolicy<OpenHumanRunContext>> =
            Arc::new(TinySecurityPolicy::with_sources(sources));
        let mut policies = vec![security_policy];
        if let Some(enforcement) = &enforcement {
            policies.push(super::tool_policy::ToolPolicyMiddleware::builder_policy(
                enforcement.policy.clone(),
                tool_sets.clone(),
            ));
        }
        let policy: Arc<dyn ToolCallPolicy<OpenHumanRunContext>> =
            Arc::new(CombinedPolicy { policies });
        let resolver = Arc::new(ApprovalSecurityMiddleware::new(tool_sets.clone()));
        let mut gate = gate(policy, resolver);
        if let Some(enforcement) = enforcement {
            gate = gate.with_session_ceiling(
                super::tool_policy::ToolPolicyMiddleware::session_ceiling(
                    enforcement.session.clone(),
                    tool_sets.clone(),
                    enforcement.channel.clone(),
                ),
            );
            gate = gate.with_post_gate(super::tool_policy::ToolPolicyMiddleware::skill_post_gate(
                enforcement.session,
                tool_sets,
                enforcement.channel,
            ));
        }
        Arc::new(gate)
    }
    #[cfg(not(feature = "security-module"))]
    {
        let _ = channel;
        let resolver = Arc::new(ApprovalSecurityMiddleware::new(tool_sets.clone()));
        let mut policies: Vec<Arc<dyn ToolCallPolicy<OpenHumanRunContext>>> =
            vec![Arc::new(MissingModulePolicy {
                effects: ApprovalSecurityMiddleware::new(tool_sets.clone()),
            })];
        if let Some(enforcement) = &enforcement {
            policies.push(super::tool_policy::ToolPolicyMiddleware::builder_policy(
                enforcement.policy.clone(),
                tool_sets.clone(),
            ));
        }
        let mut gate = gate(Arc::new(CombinedPolicy { policies }), resolver);
        if let Some(enforcement) = enforcement {
            gate = gate.with_session_ceiling(
                super::tool_policy::ToolPolicyMiddleware::session_ceiling(
                    enforcement.session.clone(),
                    tool_sets.clone(),
                    enforcement.channel.clone(),
                ),
            );
            gate = gate.with_post_gate(super::tool_policy::ToolPolicyMiddleware::skill_post_gate(
                enforcement.session,
                tool_sets,
                enforcement.channel,
            ));
        }
        Arc::new(gate)
    }
}

/// Installs the security gate on `harness`. Call it before any other tool
/// middleware so a refused call never reaches them.
///
/// `channel` is the session's channel tool policy, for a session that has one.
pub(crate) fn install(
    harness: &mut AgentHarness<(), OpenHumanRunContext>,
    tool_sets: Vec<Arc<Vec<Box<dyn Tool>>>>,
    channel: Option<tinytools::ToolPolicySession>,
    enforcement: Option<ToolPolicyEnforcement>,
) {
    harness.push_tool_middleware(tool_middleware(tool_sets, channel, enforcement));
}

#[cfg(all(test, feature = "security-module"))]
#[path = "security_gate_tests.rs"]
mod tests;
