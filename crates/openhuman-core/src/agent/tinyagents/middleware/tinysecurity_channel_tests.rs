use super::*;
use std::collections::HashMap;

use serde_json::json;
use tinyagents_harness::context::RunConfig;
use tinytools::{
    PermissionLevel, TaskProfile, TaskRiskLevel, ToolPolicyDecision, ToolPolicySession, ToolResult,
};

/// Builds a session whose channel policy yields `action` for `tool_name`.
fn policy_session(tool_name: &str, action: ToolPolicyAction) -> ToolPolicySession {
    let mut decisions = HashMap::new();
    decisions.insert(
        tool_name.to_string(),
        ToolPolicyDecision {
            tool_name: tool_name.to_string(),
            action,
            required_permission: Some(PermissionLevel::Write),
            allowed_permission: PermissionLevel::Write,
        },
    );
    ToolPolicySession {
        profile: TaskProfile {
            agent_id: "lead".to_string(),
            channel: "telegram".to_string(),
            entrypoint: "chat".to_string(),
            risk_level: TaskRiskLevel::Low,
            allowed_permission: PermissionLevel::Write,
        },
        capabilities: Vec::new(),
        allowed_tool_names: Default::default(),
        blocked_tool_names: Default::default(),
        hidden_tool_names: Default::default(),
        decisions,
    }
}

struct Plain;

#[async_trait]
impl Tool for Plain {
    fn name(&self) -> &str {
        "write_file"
    }
    fn description(&self) -> &str {
        "test tool"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({ "type": "object" })
    }
    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        unreachable!("the channel source never executes a tool")
    }
}

fn source(action: ToolPolicyAction) -> ChannelApprovalVerdicts {
    ChannelApprovalVerdicts::new(
        policy_session("write_file", action),
        vec![Arc::new(vec![Box::new(Plain) as Box<dyn Tool>])],
    )
}

fn ctx() -> RunContext<OpenHumanRunContext> {
    RunContext::new(
        RunConfig::new("channel-verdicts"),
        OpenHumanRunContext::new(),
    )
}

fn call(name: &str) -> TaToolCall {
    TaToolCall {
        id: "c1".into(),
        name: name.into(),
        arguments: json!({ "path": "notes.md" }),
        invalid: None,
    }
}

/// A `RequireApproval` channel verdict must never resolve to a bare allow.
///
/// The original mapping returned "no verdict" for it on the theory that the
/// call would fall through to an approval park; only some tools reach one, so an
/// ordinary tool was authorized with nobody asked. No approval gate is installed in
/// tests, and the channel files `RequireApproval` under blocked, so the only safe
/// answer is a denial that says why.
#[tokio::test]
async fn require_approval_never_silently_allows_a_plain_tool() {
    assert!(ApprovalGate::try_global().is_none());
    let channel = source(ToolPolicyAction::RequireApproval);
    let answer = channel.verdict(&ctx(), &call("write_file")).await.unwrap();
    assert_eq!(
        answer,
        Verdict::Deny {
            reason: DenialReason::Access
        }
    );
    let text = channel
        .denial_text(&call("write_file"), &answer)
        .expect("the refusal says approval is unavailable");
    assert!(text.contains("requires human approval"), "{text}");
    assert!(text.contains("no approval flow"), "{text}");
}

#[tokio::test]
async fn a_tool_the_channel_allows_or_does_not_mention_is_allowed() {
    let channel = source(ToolPolicyAction::Allow);
    assert_eq!(
        channel.verdict(&ctx(), &call("write_file")).await,
        Ok(Verdict::Allow)
    );
    let channel = source(ToolPolicyAction::RequireApproval);
    // The session has no decision for another tool, which the channel permits.
    let other = channel.verdict(&ctx(), &call("read_file")).await.unwrap();
    assert_eq!(other, Verdict::Allow);
}

#[tokio::test]
async fn a_channel_deny_is_left_to_the_tool_policy_middleware() {
    for action in [ToolPolicyAction::Deny, ToolPolicyAction::HideFromPrompt] {
        let channel = source(action);
        assert_eq!(
            channel.verdict(&ctx(), &call("write_file")).await,
            Ok(Verdict::Allow)
        );
    }
}

#[test]
fn only_its_own_missing_approval_flow_denial_gets_custom_text() {
    let channel = source(ToolPolicyAction::RequireApproval);
    let access = Verdict::Deny {
        reason: DenialReason::Access,
    };
    assert!(channel.denial_text(&call("write_file"), &access).is_some());
    // A tool the channel does not gate keeps the category text.
    assert!(channel.denial_text(&call("read_file"), &access).is_none());
    // So does any other verdict for a gated tool.
    for other in [
        Verdict::Allow,
        Verdict::Deny {
            reason: DenialReason::ApprovalUnavailable,
        },
    ] {
        assert!(channel.denial_text(&call("write_file"), &other).is_none());
    }
}

/// The channel's verdict folds with the module's: a module refusal wins over the
/// channel's request for a human, and the channel's request wins over an allow.
#[tokio::test]
async fn the_channel_request_folds_under_a_module_refusal() {
    use super::super::tinysecurity::{combine_decisions, verdict_to_policy_decision};
    let ask = verdict_to_policy_decision(
        "write_file",
        &Verdict::Deny {
            reason: DenialReason::ApprovalUnavailable,
        },
    );
    let refuse = verdict_to_policy_decision(
        "write_file",
        &Verdict::Deny {
            reason: DenialReason::ReadOnlyMode,
        },
    );
    assert!(matches!(
        combine_decisions(vec![ask.clone(), refuse]),
        tinyagents_harness::middleware::PolicyDecision::Deny { .. }
    ));
    assert!(matches!(
        combine_decisions(vec![
            tinyagents_harness::middleware::PolicyDecision::Allow,
            ask
        ]),
        tinyagents_harness::middleware::PolicyDecision::RequireApproval { .. }
    ));
}
