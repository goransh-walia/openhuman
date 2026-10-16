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
//! Turns, sub-agents, flows and the live voice harness all install it through
//! [`install`], so no path carries its own copy of the wiring.

use std::sync::Arc;

use tinyagents_harness::middleware::ApprovalGateMiddleware;
use tinyagents_harness::runtime::AgentHarness;
use tinytools::Tool;

use super::approval::ApprovalSecurityMiddleware;
use crate::agent::tinyagents::host::OpenHumanRunContext;

/// Installs the security gate on `harness`. Call it before any other tool
/// middleware so a refused call never reaches them.
pub(crate) fn install(
    harness: &mut AgentHarness<(), OpenHumanRunContext>,
    tool_sets: Vec<Arc<Vec<Box<dyn Tool>>>>,
) {
    #[cfg(feature = "security-module")]
    {
        use super::tinysecurity::TinySecurityPolicy;
        use super::tinysecurity_module::ModuleVerdicts;
        use crate::agent::tinyagents::policy_denial::PolicyDenial;
        use tinyagents_harness::middleware::{PolicyDecision, ToolPolicyGateMiddleware};

        let policy = TinySecurityPolicy::new(Arc::new(ModuleVerdicts::new(tool_sets.clone())));
        let resolver = Arc::new(ApprovalSecurityMiddleware::new(tool_sets));
        let gate = ToolPolicyGateMiddleware::new(Arc::new(policy))
            .with_approval_resolver(resolver)
            .with_denial_renderer(Arc::new(|call, policy, decision| match decision {
                PolicyDecision::RequireApproval { reason } => PolicyDenial::ApprovalRequired {
                    tool: &call.name,
                    policy,
                    reason,
                },
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
            }
            .render()));
        harness.push_tool_middleware(Arc::new(gate));
    }
    #[cfg(not(feature = "security-module"))]
    harness.push_tool_middleware(Arc::new(ApprovalGateMiddleware::new(
        "approval_security",
        Arc::new(ApprovalSecurityMiddleware::new(tool_sets)),
    )));
}
