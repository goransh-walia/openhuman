//! The immutable authority view the run exposes to shared tools through the
//! harness's typed state-view seam, and the readers tools use on it.

use crate::agent::turn_origin::AgentTurnOrigin;
use tinyagents_definition::SandboxMode;

/// Minimal immutable authority view exposed to shared tools through the
/// harness's typed state-view seam.
#[derive(Clone, Debug, Default)]
pub(crate) struct HostOperationContext {
    pub(crate) origin: Option<AgentTurnOrigin>,
    /// Where this run's approvals are surfaced; see
    /// [`OpenHumanRunContext::approval`](super::OpenHumanRunContext::approval).
    pub(crate) approval: crate::security::approval::ApprovalTurnContext,
    pub(crate) sandbox_mode: Option<SandboxMode>,
    /// The agent definition the run's parent execution context belongs to, for
    /// tools that act on behalf of the spawning agent when no memory identity
    /// is scoped (see `memory::tools::CallFacts::gather`).
    pub(crate) parent_agent_id: Option<String>,
    /// Who the run acts as on memory; see
    /// [`OpenHumanRunContext::memory_identity`](super::OpenHumanRunContext::memory_identity).
    pub(crate) memory_identity: Option<crate::memory::scope::MemoryIdentity>,
}

/// Reads the spawning agent's definition id from the explicit tool carrier.
/// Direct tool dispatch has no parent.
pub(crate) fn parent_agent_id_for_tool(
    context: Option<&dyn tinytools::ToolRunContext>,
) -> Option<String> {
    context
        .and_then(tinytools::ToolRunContext::host_extension)
        .and_then(|extension| {
            extension.downcast_ref::<tinyagents_harness::tool::ToolExecutionContext>()
        })
        .and_then(|context| context.state::<HostOperationContext>())
        .and_then(|host| host.parent_agent_id.clone())
}

/// Reads the run's memory identity from the explicit tool carrier. Direct tool
/// dispatch has none.
pub(crate) fn memory_identity_for_tool(
    context: Option<&dyn tinytools::ToolRunContext>,
) -> Option<crate::memory::scope::MemoryIdentity> {
    context
        .and_then(tinytools::ToolRunContext::host_extension)
        .and_then(|extension| {
            extension.downcast_ref::<tinyagents_harness::tool::ToolExecutionContext>()
        })
        .and_then(|context| context.state::<HostOperationContext>())
        .and_then(|host| host.memory_identity.clone())
}

/// Reads the calling agent's sandbox policy from the explicit tool carrier.
/// Direct tool dispatch has no agent-specific mode.
pub(crate) fn sandbox_mode_for_tool(
    context: Option<&dyn tinytools::ToolRunContext>,
) -> Option<SandboxMode> {
    context
        .and_then(tinytools::ToolRunContext::host_extension)
        .and_then(|extension| {
            extension.downcast_ref::<tinyagents_harness::tool::ToolExecutionContext>()
        })
        .and_then(|context| context.state::<HostOperationContext>())
        .and_then(|host| host.sandbox_mode)
}

/// Reads the run's approval context from the explicit tool carrier. Direct tool
/// dispatch (CLI, JSON-RPC, tests) has none, so a tool called that way sees the
/// non-interactive default.
pub(crate) fn approval_for_tool(
    context: Option<&dyn tinytools::ToolRunContext>,
) -> crate::security::approval::ApprovalTurnContext {
    context
        .and_then(tinytools::ToolRunContext::host_extension)
        .and_then(|extension| {
            extension.downcast_ref::<tinyagents_harness::tool::ToolExecutionContext>()
        })
        .and_then(|context| context.state::<HostOperationContext>())
        .map(|host| host.approval.clone())
        .unwrap_or_default()
}

/// A tool execution context for a run whose approval context is `approval`: what
/// the harness hands a tool that is dispatched inside that run. Tests drive a
/// tool through it to prove the turn's approval context reaches the tool.
#[cfg(test)]
pub(crate) fn tool_context_with_approval(
    approval: crate::security::approval::ApprovalTurnContext,
) -> tinyagents_harness::tool::ToolExecutionContext {
    let mut host = super::OpenHumanRunContext::new();
    host.approval = approval;
    let run = host.into_tinyagents(tinyagents_harness::context::RunConfig::new("approval-test"));
    tinyagents_harness::tool::ToolExecutionContext::from_run_context(
        &run,
        tinyagents_harness::ids::CallId::new("approval-test-call"),
    )
}
