use super::*;

struct ScriptedPolicy(PolicyDecision);

#[async_trait::async_trait]
impl ToolCallPolicy<OpenHumanRunContext> for ScriptedPolicy {
    fn name(&self) -> &str {
        "scripted"
    }

    async fn check(
        &self,
        _ctx: &RunContext<OpenHumanRunContext>,
        _call: &TaToolCall,
    ) -> PolicyDecision {
        self.0.clone()
    }
}

fn check_context() -> (RunContext<OpenHumanRunContext>, TaToolCall) {
    (
        RunContext::new(
            tinyagents_harness::context::RunConfig::new("feature-off-security-gate"),
            OpenHumanRunContext::new(),
        ),
        TaToolCall {
            id: "call-1".into(),
            name: "send_email".into(),
            arguments: serde_json::json!({}),
            invalid: None,
        },
    )
}

#[tokio::test]
async fn fallback_gate_denial_dominates_approval() {
    let policy = FallbackPolicy {
        policies: vec![
            Arc::new(ScriptedPolicy(PolicyDecision::require_approval("human"))),
            Arc::new(ScriptedPolicy(PolicyDecision::deny("host refusal"))),
            Arc::new(ScriptedPolicy(PolicyDecision::Allow)),
        ],
    };
    let (ctx, call) = check_context();

    assert!(matches!(
        policy.check(&ctx, &call).await,
        PolicyDecision::Deny { .. }
    ));
}

#[tokio::test]
async fn fallback_gate_approval_dominates_allow() {
    let policy = FallbackPolicy {
        policies: vec![
            Arc::new(ScriptedPolicy(PolicyDecision::Allow)),
            Arc::new(ScriptedPolicy(PolicyDecision::require_approval("human"))),
        ],
    };
    let (ctx, call) = check_context();

    assert!(matches!(
        policy.check(&ctx, &call).await,
        PolicyDecision::RequireApproval { .. }
    ));
}
