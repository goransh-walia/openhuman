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
