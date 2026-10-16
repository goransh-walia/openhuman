//! The production [`VerdictSource`]: ask the TinySecurity module whether a tool
//! call may run (issue #7328, phase 6).
//!
//! Every tool that could be refused reaches the module as a host-tool admission
//! question (`Check::HostAction`, or `Check::HostShell` for `shell`); the module
//! decides from the verified caller (origin, tier), the tool's declared permission
//! level and external effect, and the command scope registered for the user's
//! autonomy settings (its tier and hourly budget). This file derives the checks
//! from the tool the host will execute, never from model arguments, and passes the
//! module's answer through unchanged; mapping it onto the harness decision and
//! the model-facing text is [`super::tinysecurity`]'s job.
//!
//! The one answer the host does not ask for is the one the module could only give
//! one way: a tool that neither acts nor has an external effect (and is not
//! `shell`) is allowed in every tier and origin, so it runs without a bus call.
//!
//! Any failure to obtain an answer is an `Err`, which the policy turns into a
//! denial. The one exception is a process that never installed the approval gate
//! (headless embed, CLI, tests): it has no human to ask, the resolver has always
//! let such a call through, and so a module that cannot answer does not stop a
//! non-shell tool there. `shell` is always refused without an answer.

use std::sync::Arc;

use async_trait::async_trait;
use tinyagents_harness::context::RunContext;
use tinyinference_llm::tool::ToolCall as TaToolCall;
use tinysecurity_bus::{
    AccessTier, CallerContext, Check, DenialReason, Origin, ToolCall, ToolPermission, Verdict,
};
use tinytools::{PermissionLevel, Tool};

use super::approval::{approval_tool_name, ApprovalSecurityMiddleware};
use super::tinysecurity::{VerdictSource, SHELL_TOOL};
use crate::agent::tinyagents::host::OpenHumanRunContext;
use crate::agent::turn_origin::{AgentTurnOrigin, TrustedAutomationSource};
use crate::security::approval::ApprovalGate;
use crate::security::policy::{CommandFlow, CommandStage, SecurityPolicy};

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

/// The origin of the turn the call belongs to: the run's own, else the one the
/// dispatch or the calling task's turn scope carries.
fn turn_origin(ctx: &RunContext<OpenHumanRunContext>) -> Option<AgentTurnOrigin> {
    ctx.data
        .origin
        .clone()
        .or_else(|| ctx.data.dispatch_ctx().turn_origin_or_scoped())
}

/// The origin class the module sees, from the verified turn context.
fn module_origin(ctx: &RunContext<OpenHumanRunContext>) -> Origin {
    if matches!(
        turn_origin(ctx),
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

/// The module's permission vocabulary for the host's tool permission level.
fn permission(level: PermissionLevel) -> ToolPermission {
    match level {
        PermissionLevel::None => ToolPermission::None,
        PermissionLevel::ReadOnly => ToolPermission::ReadOnly,
        PermissionLevel::Write => ToolPermission::Write,
        PermissionLevel::Execute => ToolPermission::Execute,
        PermissionLevel::Dangerous => ToolPermission::Dangerous,
    }
}

/// The host-derived checks for one call, or the verdict that needs none.
enum Plan {
    /// Ask the module these checks, under the named autonomy scope.
    Ask { checks: Vec<Check>, redact: bool },
    /// The answer is known without asking.
    Settled(Verdict),
}

#[async_trait]
impl VerdictSource for ModuleVerdicts {
    async fn verdict(
        &self,
        ctx: &RunContext<OpenHumanRunContext>,
        call: &TaToolCall,
    ) -> Result<Verdict, String> {
        let dispatch = ctx.data.dispatch_ctx();
        let shell = call.name == SHELL_TOOL;
        let plan = {
            // Unknown name: nothing to reason about, and "unknown" never means fine.
            let Some(tool) = self.effects.resolve_tool(&call.name) else {
                tracing::warn!(tool = %call.name, "{LOG} denied: tool is not registered in this session");
                return Ok(Verdict::Deny {
                    reason: DenialReason::UnsupportedEffect,
                });
            };
            let level = tool.permission_level_with_args(&call.arguments);
            let external = tool.external_effect_with_args(&call.arguments);
            (level, external)
        };
        let (level, declared_external) = plan;
        // The desktop approvals-off exception only ever lifts a prompt.
        let external = declared_external
            && !self
                .effects
                .desktop_approvals_disabled(&dispatch, &call.name)
                .await;
        let plan = if shell {
            match shell_check(&call.arguments) {
                Some(check) => Plan::Ask {
                    checks: vec![check],
                    redact: false,
                },
                None => Plan::Settled(Verdict::Deny {
                    reason: DenialReason::Command,
                }),
            }
        } else if !level_acts(level) && !external {
            Plan::Settled(Verdict::Allow)
        } else {
            Plan::Ask {
                checks: vec![Check::HostAction {
                    permission: permission(level),
                    external_effect: external,
                    // An allowed acting call spends one action of the hourly budget.
                    spend: level_acts(level),
                    command_policy: None,
                }],
                redact: external,
            }
        };
        let Plan::Ask { checks, redact } = plan else {
            let Plan::Settled(verdict) = plan else {
                unreachable!("a plan is either asked or settled");
            };
            return Ok(verdict);
        };
        match self
            .ask(ctx, &dispatch, call, checks, redact)
            .await
        {
            Ok(verdict) => Ok(verdict),
            Err(fault) if !shell && ApprovalGate::try_global().is_none() => {
                tracing::warn!(
                    tool = %call.name,
                    "{LOG} no approval gate installed and the module cannot answer; not refusing a non-shell tool"
                );
                let _ = fault;
                Ok(Verdict::Allow)
            }
            Err(fault) => Err(fault),
        }
    }
}

fn level_acts(level: PermissionLevel) -> bool {
    level > PermissionLevel::ReadOnly
}

/// The shell command gate for `arguments`, or `None` when the call carries no
/// command to classify. The model's `category` can only raise the class.
fn shell_check(arguments: &serde_json::Value) -> Option<Check> {
    let command = arguments.get("command").and_then(serde_json::Value::as_str)?;
    let declared_class = arguments
        .get("category")
        .and_then(serde_json::Value::as_str)
        .and_then(SecurityPolicy::parse_declared_class)
        .map(crate::security::policy::native_commands::to_bus_class);
    Some(Check::HostShell {
        command: command.to_owned(),
        declared_class,
        // The shell tool reserves an action when it runs; asking never does.
        spend: false,
        command_policy: None,
    })
}

impl ModuleVerdicts {
    async fn ask(
        &self,
        ctx: &RunContext<OpenHumanRunContext>,
        dispatch: &crate::core::runtime::DispatchCtx,
        call: &TaToolCall,
        mut checks: Vec<Check>,
        redact: bool,
    ) -> Result<Verdict, String> {
        let config = dispatch.load_config().await?;
        // The scope the shell tool spends its hourly budget through: the same
        // settings, so an equal policy reuses the registered scope and its ledger.
        let policy = crate::security::live_policy::effective_in(dispatch)
            .unwrap_or_else(|| Arc::new(SecurityPolicy::from_full_config(&config)));
        let scope = crate::modules::security::command::command_scope_id(
            &policy.workspace_dir,
            policy
                .native_command_policy(CommandFlow::Harness, CommandStage::Execute)
                .map_err(|refusal| refusal.to_string())?,
        )
        .await?;
        for check in &mut checks {
            match check {
                Check::HostAction { command_policy, .. }
                | Check::HostShell { command_policy, .. } => *command_policy = Some(scope.clone()),
                _ => {}
            }
        }
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
        // Arguments are only evidence for a human's approval; a call that needs
        // none is judged by its checks alone.
        let arguments_json = if redact {
            serde_json::to_string(&crate::security::redact::arguments(&call.arguments))
                .map_err(|_| "argument encoding failed".to_string())?
        } else {
            "{}".to_string()
        };
        let decision = crate::modules::security::evaluate(
            &config,
            caller,
            ToolCall {
                tool,
                arguments_json,
                checks,
            },
        )
        .await?;
        Ok(decision.verdict)
    }
}
