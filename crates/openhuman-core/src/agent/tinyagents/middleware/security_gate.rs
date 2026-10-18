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

use tinyagents_harness::runtime::AgentHarness;
use tinytools::Tool;

use super::approval::ApprovalSecurityMiddleware;
use crate::agent::tinyagents::host::OpenHumanRunContext;

/// The gate over `policy`, settling a required approval through `resolver`.
/// Denials render in the structured `Blocked / Reason / Workaround` shape the
/// rest of the host's policy boundaries use.
#[cfg(feature = "security-module")]
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
) -> Arc<dyn tinyagents_harness::middleware::ToolMiddleware<(), OpenHumanRunContext>> {
    #[cfg(feature = "security-module")]
    {
        use super::tinysecurity::{TinySecurityPolicy, VerdictSource};
        use super::tinysecurity_channel::ChannelApprovalVerdicts;
        use super::tinysecurity_module::ModuleVerdicts;

        let mut sources: Vec<Arc<dyn VerdictSource>> =
            vec![Arc::new(ModuleVerdicts::new(tool_sets.clone()))];
        if let Some(session) = channel {
            sources.push(Arc::new(ChannelApprovalVerdicts::new(
                session,
                tool_sets.clone(),
            )));
        }
        let policy = TinySecurityPolicy::with_sources(sources);
        let resolver = Arc::new(ApprovalSecurityMiddleware::new(tool_sets));
        Arc::new(gate(Arc::new(policy), resolver))
    }
    #[cfg(not(feature = "security-module"))]
    {
        let _ = channel;
        Arc::new(tinyagents_harness::middleware::ApprovalGateMiddleware::new(
            "approval_security",
            Arc::new(ApprovalSecurityMiddleware::new(tool_sets)),
        ))
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
) {
    harness.push_tool_middleware(tool_middleware(tool_sets, channel));
}

#[cfg(all(test, feature = "security-module"))]
#[path = "security_gate_tests.rs"]
mod tests;
