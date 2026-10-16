use super::*;

fn is_deny(d: &PolicyDecision) -> bool {
    matches!(d, PolicyDecision::Deny { .. })
}

#[test]
fn allow_maps_to_allow() {
    assert!(matches!(
        verdict_to_policy_decision(&Verdict::Allow),
        PolicyDecision::Allow
    ));
}

#[test]
fn deny_maps_to_deny_without_payload() {
    let d = verdict_to_policy_decision(&Verdict::Deny {
        reason: DenialReason::Floor,
    });
    assert!(is_deny(&d));
}

#[test]
fn require_approval_carries_request_id() {
    let d = verdict_to_policy_decision(&Verdict::RequireApproval {
        request_id: "req-1".into(),
    });
    match d {
        PolicyDecision::RequireApproval { reason } => assert!(reason.contains("req-1")),
        _ => panic!("expected RequireApproval"),
    }
}

#[test]
fn deny_dominates_other_decisions() {
    let combined = combine_decisions(vec![
        PolicyDecision::Allow,
        PolicyDecision::RequireApproval { reason: "r".into() },
        PolicyDecision::Deny { reason: "d".into() },
        PolicyDecision::Allow,
    ]);
    assert!(is_deny(&combined));
}

#[test]
fn require_approval_beats_allow() {
    let combined = combine_decisions(vec![
        PolicyDecision::Allow,
        PolicyDecision::RequireApproval { reason: "r".into() },
    ]);
    assert!(matches!(combined, PolicyDecision::RequireApproval { .. }));
}

#[test]
fn empty_combination_allows() {
    assert!(matches!(combine_decisions(vec![]), PolicyDecision::Allow));
}

fn test_ctx() -> RunContext<OpenHumanRunContext> {
    RunContext::new(
        tinyagents_harness::context::RunConfig::new("tinysecurity-test"),
        OpenHumanRunContext::new(),
    )
}

fn test_call() -> TaToolCall {
    TaToolCall {
        id: "c1".into(),
        name: "shell".into(),
        arguments: serde_json::json!({}),
        invalid: None,
    }
}

/// A scripted module: answers each call from a fixed script and counts asks.
struct Scripted {
    answer: Result<Verdict, String>,
    asked: Arc<std::sync::atomic::AtomicUsize>,
}

impl Scripted {
    fn new(answer: Result<Verdict, String>) -> (Arc<Self>, Arc<std::sync::atomic::AtomicUsize>) {
        let asked = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        (
            Arc::new(Self {
                answer,
                asked: asked.clone(),
            }),
            asked,
        )
    }
}

#[async_trait]
impl VerdictSource for Scripted {
    async fn verdict(
        &self,
        _ctx: &RunContext<OpenHumanRunContext>,
        _call: &TaToolCall,
    ) -> Result<Verdict, String> {
        self.asked.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.answer.clone()
    }
}

#[tokio::test]
async fn module_fault_denies_without_retry() {
    let (source, asked) = Scripted::new(Err("module crashed".to_string()));
    let policy = TinySecurityPolicy::new(source);
    let decision = policy.check(&test_ctx(), &test_call()).await;
    assert!(is_deny(&decision), "a module fault must never allow");
    assert_eq!(
        asked.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "no retry"
    );
}

#[tokio::test]
async fn policy_maps_require_approval_verdict() {
    let (source, _) = Scripted::new(Ok(Verdict::RequireApproval {
        request_id: "req-9".into(),
    }));
    let policy = TinySecurityPolicy::new(source);
    match policy.check(&test_ctx(), &test_call()).await {
        PolicyDecision::RequireApproval { reason } => assert!(reason.contains("req-9")),
        other => panic!("expected RequireApproval, got {other:?}"),
    }
}

#[test]
fn the_modules_human_needed_answer_requires_approval_and_never_allows() {
    let decision = verdict_to_policy_decision(&Verdict::Deny {
        reason: DenialReason::ApprovalUnavailable,
    });
    assert!(matches!(decision, PolicyDecision::RequireApproval { .. }));
}

#[test]
fn every_other_denial_stays_a_denial() {
    for reason in [
        DenialReason::Floor,
        DenialReason::UnsupportedEffect,
        DenialReason::Access,
        DenialReason::PathScope,
        DenialReason::PathResolution,
        DenialReason::Command,
        DenialReason::InvalidContext,
        DenialReason::Content,
    ] {
        let decision = verdict_to_policy_decision(&Verdict::Deny { reason });
        assert!(is_deny(&decision), "{decision:?}");
    }
}

#[tokio::test]
async fn several_sources_fold_with_deny_over_approval_over_allow() {
    let allow = Scripted::new(Ok(Verdict::Allow)).0;
    let approve = Scripted::new(Ok(Verdict::RequireApproval {
        request_id: "r".into(),
    }))
    .0;
    let deny = Scripted::new(Ok(Verdict::Deny {
        reason: DenialReason::Floor,
    }))
    .0;

    let both = TinySecurityPolicy::with_sources(vec![allow.clone(), approve.clone()]);
    assert!(matches!(
        both.check(&test_ctx(), &test_call()).await,
        PolicyDecision::RequireApproval { .. }
    ));
    let all = TinySecurityPolicy::with_sources(vec![allow, approve, deny]);
    assert!(is_deny(&all.check(&test_ctx(), &test_call()).await));
}

#[tokio::test]
async fn a_fault_in_one_source_denies_and_skips_the_rest() {
    let (fault, _) = Scripted::new(Err("down".to_string()));
    let (later, later_asked) = Scripted::new(Ok(Verdict::Allow));
    let policy = TinySecurityPolicy::with_sources(vec![fault, later]);
    assert!(is_deny(&policy.check(&test_ctx(), &test_call()).await));
    assert_eq!(later_asked.load(std::sync::atomic::Ordering::SeqCst), 0);
}
