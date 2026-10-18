use super::*;

struct Nobody;

#[async_trait]
impl PromptSurface for Nobody {
    async fn deliver(&self, _delivery: Delivery) -> Result<(), String> {
        Err("no surface".into())
    }
}

fn input() -> CheckInput {
    CheckInput {
        tenant: Tenant::default(),
        variant: Variant::Standard,
        posture: Posture::default(),
        call_id: "00000001".into(),
        tool: "gmail_send".into(),
        args: serde_json::json!({}),
        summary: "send".into(),
        thread: None,
        flow: None,
        origin: CallOrigin::Operator,
        ttl: Some(Duration::from_secs(600)),
        prompt: Arc::new(Nobody),
    }
}

#[test]
fn each_posture_variant_is_its_own_subject() {
    let suffixes = [
        Variant::Standard.suffix(),
        Variant::FlowStrict.suffix(),
        Variant::Forced.suffix(),
    ];
    assert_eq!(suffixes[0], "");
    assert_ne!(suffixes[1], suffixes[2]);
    assert!(suffixes[1].starts_with('#') && suffixes[2].starts_with('#'));
}

#[test]
fn the_default_posture_requires_approval_for_nothing_it_does_not_name() {
    let posture = Posture::default();
    assert!(!posture.auto_approve_all);
    assert!(posture.tool_grants.is_empty());
}

#[tokio::test]
async fn a_build_without_the_module_refuses_every_gated_call() {
    let engine = UnavailableEngine;
    assert!(matches!(
        engine.check(input()).await,
        Err(EngineError::Unavailable(message)) if message.contains("security-module")
    ));
    assert!(engine.reserve("r").await.is_err());
    assert!(engine
        .record_execution("r", super::super::types::ExecutionOutcome::Success)
        .await
        .is_err());
    assert!(engine.expire("r").await.is_err());
    assert!(engine.cancel("r").await.is_err());
    // Nothing is pending, so a decision finds nothing; history is empty, not an error.
    assert_eq!(
        engine.decide("r", Choice::Once, "approve_once").await,
        Err(EngineError::NotPending)
    );
    assert_eq!(engine.is_decided("r").await, Ok(None));
    assert!(engine.history(5).await.unwrap().is_empty());
    engine.release("r");
}
