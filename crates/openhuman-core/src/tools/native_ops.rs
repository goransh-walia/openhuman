use std::sync::Arc;
use std::time::Instant;

use crate::agent::host_runtime::{NativeRuntime, RuntimeAdapter};
use crate::agent::tinyagents::host::OpenHumanRunContext;
use crate::config::Config;
use crate::core::bus::BUS;
use crate::core::events::DomainEvent;
use crate::security::SecurityPolicy;
use crate::tools;
use crate::tools::native_ops_types::{ExecuteToolOutcome, RuntimeToolSummary};
use tinyagents_harness::context::{RunConfig, RunContext};
use tinyagents_harness::middleware::{
    BoxToolFuture, MiddlewareStack, ToolBaseCall, ToolMiddleware,
};
use tinybox_bus::CommandClass;
use tinyinference_llm::tool::ToolCall;
use tinytools::{Tool, ToolCallOptions, ToolResult, ToolScope};
use tracing::{debug, trace};

fn tool_scope_label(scope: ToolScope) -> &'static str {
    match scope {
        ToolScope::All => "all",
        ToolScope::AgentOnly => "agent_only",
        ToolScope::CliRpcOnly => "cli_rpc_only",
    }
}

fn summarize_tool(tool: &dyn Tool) -> RuntimeToolSummary {
    RuntimeToolSummary {
        name: tool.name().to_string(),
        description: tool.description().to_string(),
        category: tool.category().to_string(),
        permission_level: tool.permission_level().to_string(),
        scope: tool_scope_label(tool.scope()).to_string(),
        supports_markdown: tool.supports_markdown(),
        parameters: tool.parameters_schema(),
    }
}

fn classify_shell_tool_call(args: &serde_json::Value) -> CommandClass {
    // This synchronous summary API cannot ask TinyBox to analyze the command.
    // Report the conservative class here; the invocation path performs exact
    // module analysis before approval and execution.
    let mut class = CommandClass::Write;
    if let Some(declared) = args
        .get("category")
        .and_then(|value| value.as_str())
        .and_then(SecurityPolicy::parse_declared_class)
    {
        class = class.max(declared);
    }
    class
}

/// The command class of one tool call. `shell` is classified from its declared
/// category (the invocation path runs the module's exact command analysis before
/// approval and execution); every other tool is classified by the module from its
/// permission level and external effect (`ClassifyHostAction2`).
async fn command_class_for_tool(
    tool: &dyn Tool,
    args: &serde_json::Value,
) -> Result<CommandClass, String> {
    if tool.name() == "shell" {
        return Ok(classify_shell_tool_call(args));
    }
    crate::security::policy::host_action_class(
        tool.permission_level_with_args(args),
        tool.external_effect_with_args(args),
    )
    .await
}

pub fn build_runtime_tools(config: &Config) -> Result<Vec<Box<dyn Tool>>, String> {
    debug!(
        workspace = %config.workspace_dir.display(),
        "[native_ops] build_runtime_tools: start"
    );
    let security = Arc::new(SecurityPolicy::from_full_config(config));
    // Phase 1 of #1401: see comment in channels/runtime/startup.rs.
    let audit = crate::security::get_or_create_workspace_audit_logger(
        config.security.audit.clone(),
        config.workspace_dir.clone(),
    )
    .map_err(|e| e.to_string())?;
    let runtime: Arc<dyn RuntimeAdapter> = Arc::new(NativeRuntime::new());
    trace!("[native_ops] build_runtime_tools: tools::ops::all_tools_with_runtime");
    let built = tools::ops::all_tools_with_runtime(
        Arc::new(config.clone()),
        &security,
        runtime,
        audit,
        &config.browser,
        &config.http_request,
        &config.action_dir,
        &config.agents,
        config,
        None,
    );
    debug!(
        tool_count = built.len(),
        "[native_ops] build_runtime_tools: done"
    );
    Ok(built)
}

pub fn list_tools(config: &Config) -> Result<Vec<RuntimeToolSummary>, String> {
    debug!("[native_ops] list_tools: start");
    let mut summaries: Vec<RuntimeToolSummary> = build_runtime_tools(config)?
        .into_iter()
        .map(|tool| summarize_tool(tool.as_ref()))
        .collect();
    summaries.sort_by(|a, b| a.name.cmp(&b.name));
    debug!(count = summaries.len(), "[native_ops] list_tools: done");
    Ok(summaries)
}

pub async fn classify_tool_call(
    config: &Config,
    tool_name: &str,
    args: &serde_json::Value,
) -> Result<CommandClass, String> {
    debug!(tool_name, "[native_ops] classify_tool_call: start");
    let tools = build_runtime_tools(config)?;
    let tool = tools
        .into_iter()
        .find(|tool| tool.name() == tool_name)
        .ok_or_else(|| {
            debug!(tool_name, "[native_ops] classify_tool_call: tool not found");
            format!("unknown tool `{tool_name}`")
        })?;
    let class = command_class_for_tool(tool.as_ref(), args).await?;
    debug!(
        tool_name,
        ?class,
        permission = %tool.permission_level_with_args(args),
        external_effect = tool.external_effect_with_args(args),
        "[native_ops] classify_tool_call: done"
    );
    Ok(class)
}

/// Run one registered tool on behalf of a direct caller (the JS and RPC tool
/// runs, workflow `tool_call` nodes). The call passes the same security gate a
/// harness turn installs (`middleware::security_gate`), so a tool the policy
/// refuses never runs here either, and a call that needs a human settles through
/// the same approval resolver.
pub async fn execute_tool(
    config: &Config,
    tool_name: &str,
    args: serde_json::Value,
    prefer_markdown: bool,
) -> Result<ExecuteToolOutcome, String> {
    debug!(
        tool_name,
        prefer_markdown, "[native_ops] execute_tool: start"
    );
    let tools: Arc<Vec<Box<dyn Tool>>> = Arc::new(build_runtime_tools(config)?);
    trace!(
        tool_count = tools.len(),
        tool_name,
        "[native_ops] execute_tool: runtime tools built"
    );
    let gate = crate::agent::tinyagents::middleware::security_gate::tool_middleware(
        vec![Arc::clone(&tools)],
        None,
    );
    run_gated(&tools, gate, tool_name, args, prefer_markdown).await
}

/// The tool, run once the middleware has judged the call. The base of the
/// middleware stack: reached only when the gate allowed the call.
struct DirectTool<'a> {
    tool: &'a dyn Tool,
    prefer_markdown: bool,
    /// The tool's own error, kept whole for the caller: the harness result type
    /// carries text only.
    failure: std::sync::Mutex<Option<String>>,
}

impl ToolBaseCall<(), OpenHumanRunContext> for DirectTool<'_> {
    fn call<'a>(
        &'a self,
        _ctx: &'a RunContext<OpenHumanRunContext>,
        _state: &'a (),
        call: ToolCall,
    ) -> BoxToolFuture<'a> {
        Box::pin(async move {
            match self
                .tool
                .execute_with_options(
                    call.arguments,
                    ToolCallOptions {
                        prefer_markdown: self.prefer_markdown,
                    },
                )
                .await
            {
                Ok(result) => Ok(result),
                Err(error) => {
                    let message = format!("{error:#}");
                    if let Ok(mut failure) = self.failure.lock() {
                        *failure = Some(message.clone());
                    }
                    Ok(ToolResult::error(message))
                }
            }
        })
    }
}

/// [`execute_tool`] over an explicit gate, so the gating is testable without a
/// loaded security module.
pub(crate) async fn run_gated(
    tools: &[Box<dyn Tool>],
    gate: Arc<dyn ToolMiddleware<(), OpenHumanRunContext>>,
    tool_name: &str,
    args: serde_json::Value,
    prefer_markdown: bool,
) -> Result<ExecuteToolOutcome, String> {
    let tool = tools
        .iter()
        .find(|tool| tool.name() == tool_name)
        .ok_or_else(|| {
            debug!(tool_name, "[native_ops] execute_tool: tool not found");
            format!("unknown tool `{tool_name}`")
        })?;

    let started = Instant::now();
    debug!(
        tool_name,
        "[native_ops] execute_tool: publish ToolExecutionStarted"
    );
    BUS.publish(DomainEvent::ToolExecutionStarted {
        tool_name: tool_name.to_string(),
        session_id: "javascript".to_string(),
    });

    let mut stack: MiddlewareStack<(), OpenHumanRunContext> = MiddlewareStack::new();
    stack.push_tool_middleware(gate);
    let ctx = RunContext::new(RunConfig::new("direct-tool"), OpenHumanRunContext::new());
    let call = ToolCall {
        id: format!("direct-{tool_name}"),
        name: tool_name.to_string(),
        arguments: args,
        invalid: None,
    };
    let base = DirectTool {
        tool: tool.as_ref(),
        prefer_markdown,
        failure: std::sync::Mutex::new(None),
    };
    trace!(
        tool_name,
        "[native_ops] execute_tool: dispatch through the security gate"
    );
    let execution = stack
        .run_wrapped_tool(&ctx, &(), call, &base)
        .await
        .map(|outcome| outcome.into_result())
        .map_err(|error| format!("tool `{tool_name}` failed: {error}"))
        .and_then(|result| match base.failure.lock().ok().and_then(|f| f.clone()) {
            Some(error) => {
                debug!(tool_name, error = %error, "[native_ops] execute_tool: tool execution failed");
                Err(format!("tool `{tool_name}` failed: {error}"))
            }
            None => Ok(result),
        });

    let elapsed_ms = started.elapsed().as_millis() as u64;
    let success = execution
        .as_ref()
        .map(|result| !result.is_error)
        .unwrap_or(false);
    debug!(
        tool_name,
        success, elapsed_ms, "[native_ops] execute_tool: publish ToolExecutionCompleted"
    );
    BUS.publish(DomainEvent::ToolExecutionCompleted {
        tool_name: tool_name.to_string(),
        session_id: "javascript".to_string(),
        success,
        elapsed_ms,
    });

    let result = execution?;
    trace!(
        tool_name,
        success = !result.is_error,
        elapsed_ms,
        "[native_ops] execute_tool: returning outcome"
    );
    Ok(ExecuteToolOutcome {
        tool_name: tool_name.to_string(),
        elapsed_ms,
        result,
    })
}

#[cfg(all(test, feature = "security-module"))]
#[path = "native_ops_tests.rs"]
mod tests;
