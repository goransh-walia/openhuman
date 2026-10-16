//! The production [`VerdictSource`]: ask the TinySecurity module whether a tool
//! call may run (issue #7328, phase 6).
//!
//! Only effect-bearing host tools reach the module. A tool that declares no
//! external effect for these arguments (and a desktop tool whose approvals the
//! user turned off) is allowed here without a bus call, exactly as the approval
//! resolver always treated it; the module answers `UnsupportedEffect` for a
//! plain tool that carries no `Check::HostTool`, so asking would only deny it.
//!
//! The checks are derived here from the tool the host will execute, never from
//! model arguments. The module's answer is passed through unchanged; mapping it
//! onto the harness decision is [`super::tinysecurity`]'s job. Any failure to
//! obtain an answer is an `Err`, which the policy turns into a denial.

use std::sync::Arc;

use async_trait::async_trait;
use tinyagents_harness::context::RunContext;
use tinyinference_llm::tool::ToolCall as TaToolCall;
use tinysecurity_bus::{AccessTier, CallerContext, Check, Origin, ToolCall, Verdict};
use tinytools::Tool;

use super::approval::{approval_tool_name, ApprovalSecurityMiddleware};
use super::tinysecurity::VerdictSource;
use crate::agent::tinyagents::host::OpenHumanRunContext;
use crate::agent::turn_origin::{AgentTurnOrigin, TrustedAutomationSource};
use crate::security::approval::ApprovalGate;

const LOG: &str = "[tinyagents::mw::tinysecurity]";

/// Asks the module through `modules::security::evaluate`.
pub(crate) struct ModuleVerdicts {
    /// Resolves a call's tool and its external-effect declaration, and applies
    /// the desktop approvals-off exception the approval resolver always applied.
    effects: ApprovalSecurityMiddleware,
}

impl ModuleVerdicts {
    pub(crate) fn new(tool_sets: Vec<Arc<Vec<Box<dyn Tool>>>>) -> Self {
        Self {
            effects: ApprovalSecurityMiddleware::new(tool_sets),
        }
    }
}

/// The origin class the module sees, from the verified turn context.
fn module_origin(ctx: &RunContext<OpenHumanRunContext>) -> Origin {
    if matches!(
        ctx.data.origin,
        Some(AgentTurnOrigin::TrustedAutomation {
            source: TrustedAutomationSource::Cron,
            ..
        })
    ) {
        Origin::Cron
    } else if crate::core::runtime::is_saas() {
        Origin::Saas
    } else if ctx.data.approval.copilot_stream {
        Origin::Copilot
    } else if ctx.data.spawn_depth > 0 {
        Origin::SubAgent
    } else {
        Origin::Operator
    }
}

#[async_trait]
impl VerdictSource for ModuleVerdicts {
    async fn verdict(
        &self,
        ctx: &RunContext<OpenHumanRunContext>,
        call: &TaToolCall,
    ) -> Result<Verdict, String> {
        let dispatch = ctx.data.dispatch_ctx();
        if !self
            .effects
            .requires_approval(&dispatch, &call.name, &call.arguments)
            .await
        {
            return Ok(Verdict::Allow);
        }
        // Without an approval surface (headless embed, CLI, tests) there is no
        // human to ask and the resolver has always let the call through; the
        // module is not consulted for a process that never installed the gate.
        if ApprovalGate::try_global().is_none() {
            tracing::warn!(tool = %call.name, "{LOG} no approval gate installed; not consulting the module");
            return Ok(Verdict::Allow);
        }
        let config = dispatch.load_config().await?;
        let tenant = dispatch
            .tenant()
            .map_err(|_| "no authenticated tenant scope".to_string())?;
        let agent = tenant
            .agent
            .filter(|agent| !agent.trim().is_empty())
            .unwrap_or_else(|| "orchestrator".to_string());
        let call_id = if call.id.trim().is_empty() {
            format!("tool-{}", call.name)
        } else {
            call.id.clone()
        };
        let caller = crate::modules::security::VerifiedCaller::from_host(CallerContext {
            agent,
            thread: ctx.config.thread_id.as_ref().map(ToString::to_string),
            flow: ctx
                .data
                .approval
                .flow_run
                .as_ref()
                .map(|run| run.flow_id.clone()),
            call_id,
            origin: module_origin(ctx),
            tier: AccessTier::Full,
        })?;
        let tool = approval_tool_name(&call.name, &call.arguments).into_owned();
        let arguments_json =
            serde_json::to_string(&crate::security::redact::arguments(&call.arguments))
                .map_err(|_| "argument encoding failed".to_string())?;
        let decision = crate::modules::security::evaluate(
            &config,
            caller,
            ToolCall {
                tool,
                arguments_json,
                checks: vec![Check::HostTool {}],
            },
        )
        .await?;
        Ok(decision.verdict)
    }
}
