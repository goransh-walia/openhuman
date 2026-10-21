//! Tests for `Deliver` and `ClassifySalience`: the ack semantics of each
//! triage outcome, digests, classification and its reuse on delivery.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use tinyeventqueues_bus::{EventKind, Sender};

use super::*;
use crate::inbound::events;
use crate::inbound::fixture::{decision, decision_outcome, deferred, ScriptedTriage, Step};
use crate::triage::TriggerEnvelope;

const NOW_MS: i64 = 1_000_000;

fn now() -> i64 {
    NOW_MS
}

fn deliverer(triage: &Arc<ScriptedTriage>) -> Deliverer {
    Deliverer::new(
        triage.clone() as Arc<dyn TriageRunner>,
        DispatchCtx::host_default(),
        Duration::from_millis(200),
    )
    .with_clock(now)
    .with_max_attempts(|_| Some(3))
}

fn trigger_event(uuid: &str) -> InboundEvent {
    let envelope = TriggerEnvelope::from_composio(
        "github",
        "GITHUB_ISSUE_OPENED",
        "",
        uuid,
        json!({ "title": "flaky test" }),
    );
    events::composio(&envelope).unwrap()
}

fn turn(events: Vec<InboundEvent>, lane: Lane, attempt: u32) -> DeliverRequest {
    let mut turn = Turn::new("turn-1", lane, "c", events);
    turn.attempt = attempt;
    DeliverRequest::new(turn)
}

#[test]
fn every_action_has_a_salience_class() {
    assert_eq!(class_of(TriageAction::Drop), SalienceClass::Noise);
    assert_eq!(class_of(TriageAction::Acknowledge), SalienceClass::Fyi);
    assert_eq!(class_of(TriageAction::React), SalienceClass::Actionable);
    assert_eq!(class_of(TriageAction::Escalate), SalienceClass::Urgent);
}

#[tokio::test]
async fn a_decision_is_accepted_and_applied() {
    let triage = ScriptedTriage::new([decision(TriageAction::React)]);
    let response = deliverer(&triage)
        .deliver(turn(vec![trigger_event("u1")], Lane::Background, 1))
        .await;
    assert_eq!(response.status, DeliverStatus::Accepted);
    let applied = triage.until_applied(1, Duration::from_secs(5)).await;
    assert_eq!(
        applied,
        vec!["composio/github/GITHUB_ISSUE_OPENED".to_owned()]
    );
}

#[tokio::test]
async fn a_deferral_is_retried_when_it_asked_to_run_again() {
    let triage = ScriptedTriage::new([Step::Answer(TriageOutcome::Deferred {
        defer_until_ms: NOW_MS + 5_000,
        reason: "busy".into(),
    })]);
    let response = deliverer(&triage)
        .deliver(turn(vec![trigger_event("u1")], Lane::Background, 1))
        .await;
    assert_eq!(
        response.status,
        DeliverStatus::Retry {
            after_ms: Some(5_000)
        }
    );
    assert!(triage.applied().is_empty());
}

#[tokio::test]
async fn a_deferral_already_due_is_retried_at_once() {
    let triage = ScriptedTriage::new([Step::Answer(TriageOutcome::Deferred {
        defer_until_ms: NOW_MS - 1,
        reason: "late".into(),
    })]);
    let response = deliverer(&triage)
        .deliver(turn(vec![trigger_event("u1")], Lane::Background, 3))
        .await;
    assert_eq!(response.status, DeliverStatus::Retry { after_ms: Some(0) });
}

#[tokio::test]
async fn a_terminal_triage_is_rejected() {
    let triage = ScriptedTriage::new([Step::Answer(TriageOutcome::Terminal {
        reason: "no fallback".into(),
    })]);
    let response = deliverer(&triage)
        .deliver(turn(vec![trigger_event("u1")], Lane::Actionable, 1))
        .await;
    assert_eq!(
        response.status,
        DeliverStatus::Rejected {
            detail: "no fallback".into()
        }
    );
}

#[tokio::test]
async fn a_failed_or_slow_triage_backs_off() {
    let triage = ScriptedTriage::new([Step::Fail("provider down"), Step::Hang]);
    let deliverer = deliverer(&triage);
    for attempt in [1, 3] {
        let response = deliverer
            .deliver(turn(vec![trigger_event("u1")], Lane::Background, attempt))
            .await;
        assert_eq!(response.status, DeliverStatus::Retry { after_ms: None });
    }
    assert_eq!(triage.runs(), 2);
}

#[tokio::test]
async fn a_turn_without_a_trigger_is_rejected() {
    let triage = ScriptedTriage::new(Vec::<Step>::new());
    let deliverer = deliverer(&triage);
    let empty = deliverer.deliver(turn(vec![], Lane::Background, 1)).await;
    assert!(matches!(empty.status, DeliverStatus::Rejected { .. }));
    let chat = InboundEvent::new(
        "channel:telegram",
        "1",
        "c",
        Sender::new("u"),
        EventKind::Message,
        0,
    );
    let foreign = deliverer
        .deliver(turn(vec![chat], Lane::Interactive, 1))
        .await;
    assert!(matches!(foreign.status, DeliverStatus::Rejected { .. }));
    assert_eq!(triage.runs(), 0);
}

#[tokio::test]
async fn a_digest_is_triaged_once_for_all_its_events() {
    let triage = ScriptedTriage::new([decision(TriageAction::Escalate)]);
    let response = deliverer(&triage)
        .deliver(turn(
            vec![trigger_event("u1"), trigger_event("u2")],
            Lane::Digest,
            1,
        ))
        .await;
    assert_eq!(response.status, DeliverStatus::Accepted);
    assert_eq!(triage.runs(), 1);
    let applied = triage.until_applied(1, Duration::from_secs(5)).await;
    assert_eq!(applied, vec!["external/inbound-digest/turn-1".to_owned()]);
}

#[tokio::test]
async fn a_classification_is_reused_by_the_delivery() {
    let triage = ScriptedTriage::new([decision(TriageAction::Escalate)]);
    let deliverer = deliverer(&triage);
    let event = trigger_event("u1");
    let classified = deliverer
        .classify(ClassifySalienceRequest::new(event.clone(), 40))
        .await
        .unwrap();
    assert_eq!(classified.class, SalienceClass::Urgent);
    let response = deliverer
        .deliver(turn(vec![event], Lane::Interactive, 1))
        .await;
    assert_eq!(response.status, DeliverStatus::Accepted);
    assert_eq!(
        triage.runs(),
        1,
        "the delivery triaged a classified event again"
    );
}

#[tokio::test]
async fn a_classification_without_a_decision_keeps_the_score() {
    let triage = ScriptedTriage::new([deferred(10), Step::Fail("down"), Step::Hang]);
    let deliverer = deliverer(&triage);
    for _ in 0..3 {
        let refused = deliverer
            .classify(ClassifySalienceRequest::new(trigger_event("u1"), 40))
            .await;
        assert!(refused.is_err());
    }
    let chat = InboundEvent::new(
        "web:chat",
        "1",
        "c",
        Sender::new("u"),
        EventKind::Message,
        0,
    );
    assert!(deliverer
        .classify(ClassifySalienceRequest::new(chat, 40))
        .await
        .is_err());
    assert!(format!("{deliverer:?}").contains("Deliverer"));
}

#[test]
fn classifications_are_bounded() {
    let mut classified = Classified::default();
    let run = match decision_outcome(TriageAction::Drop) {
        TriageOutcome::Decision(run) => run,
        _ => unreachable!(),
    };
    for n in 0..=CLASSIFIED_CAPACITY {
        classified.keep(format!("k{n}"), run.clone(), NOW_MS);
    }
    classified.keep("k1".to_owned(), run.clone(), NOW_MS);
    assert_eq!(classified.runs.len(), CLASSIFIED_CAPACITY);
    assert!(
        classified.take("k0", NOW_MS).is_none(),
        "the oldest was not evicted"
    );
    assert!(classified.take("k1", NOW_MS).is_some());
    assert!(classified.take("k1", NOW_MS).is_none());
    // A stale classification is not applied on a late delivery.
    classified.keep("k2".to_owned(), run, NOW_MS);
    assert!(classified
        .take("k2", NOW_MS + CLASSIFIED_TTL_MS + 1)
        .is_none());
}

#[tokio::test]
async fn a_redelivered_accepted_turn_is_not_decided_twice() {
    // The module's callback deadline passed while the first triage was still
    // deciding; it redelivers the same turn. One decision, one apply.
    let triage =
        ScriptedTriage::new([decision(TriageAction::React), decision(TriageAction::React)]);
    let deliverer = deliverer(&triage);
    for attempt in [1, 2] {
        let response = deliverer
            .deliver(turn(vec![trigger_event("u1")], Lane::Background, attempt))
            .await;
        assert_eq!(response.status, DeliverStatus::Accepted);
    }
    assert_eq!(triage.runs(), 1, "the redelivery was triaged again");
    let _ = triage.until_applied(1, Duration::from_secs(5)).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(
        triage.applied().len(),
        1,
        "the redelivery was applied again"
    );
}

#[test]
fn accepted_turns_are_bounded() {
    let mut accepted = Accepted::default();
    for n in 0..=ACCEPTED_CAPACITY {
        accepted.remember(&format!("t{n}"));
    }
    accepted.remember("t5");
    assert_eq!(accepted.order.len(), ACCEPTED_CAPACITY);
    assert!(!accepted.contains("t0"));
    assert!(accepted.contains("t5"));
}

#[tokio::test]
async fn the_default_deliverer_reads_attempts_from_the_configured_queues() {
    let triage = ScriptedTriage::new([Step::Fail("down")]);
    let deliverer = Deliverer::new(
        triage.clone() as Arc<dyn TriageRunner>,
        DispatchCtx::host_default(),
        Duration::from_millis(100),
    );
    let response = deliverer
        .deliver(turn(vec![trigger_event("u1")], Lane::Background, 99))
        .await;
    assert_eq!(response.status, DeliverStatus::Retry { after_ms: None });
}

#[tokio::test]
async fn a_failed_apply_is_retried_instead_of_acknowledged() {
    // The agent dispatch fails: the turn must stay recoverable.
    let triage =
        ScriptedTriage::new([decision(TriageAction::React), decision(TriageAction::React)]);
    triage.fail_next_applies(1);
    let deliverer = deliverer(&triage);
    let first = deliverer
        .deliver(turn(vec![trigger_event("u1")], Lane::Background, 1))
        .await;
    assert_eq!(first.status, DeliverStatus::Retry { after_ms: None });
    assert!(triage.applied().is_empty());
    // The redelivery is not treated as already accepted: it decides and
    // applies again.
    let second = deliverer
        .deliver(turn(vec![trigger_event("u1")], Lane::Background, 2))
        .await;
    assert_eq!(second.status, DeliverStatus::Accepted);
    assert_eq!(triage.applied().len(), 1);
    assert_eq!(triage.runs(), 2);
}

#[tokio::test]
async fn a_long_apply_is_acknowledged_at_the_budget_and_finishes_on_its_own() {
    let triage = ScriptedTriage::new([decision(TriageAction::Escalate)]);
    triage.slow_applies(Duration::from_millis(300));
    let deliverer = deliverer(&triage).with_budget(Duration::from_millis(50));
    let started = std::time::Instant::now();
    let response = deliverer
        .deliver(turn(vec![trigger_event("u1")], Lane::Background, 1))
        .await;
    assert_eq!(response.status, DeliverStatus::Accepted);
    assert!(started.elapsed() < Duration::from_millis(300));
    let applied = triage.until_applied(1, Duration::from_secs(5)).await;
    assert_eq!(applied.len(), 1);
}

#[tokio::test]
async fn a_configured_deliverer_reads_the_triage_timeout_on_first_use() {
    // The first callback loads `[inbound]`: keep it off the developer's config.
    let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    let _workspace = crate::config::test_env::EnvVarGuard::workspace_unlocked(tmp.path());
    let _home = crate::config::test_env::EnvVarGuard::set("HOME", tmp.path());
    let triage = ScriptedTriage::new([decision(TriageAction::React)]);
    let deliverer = Deliverer::configured(
        triage.clone() as Arc<dyn TriageRunner>,
        DispatchCtx::host_default(),
    );
    assert!(format!("{deliverer:?}").contains("None"), "read before use");
    let response = deliverer
        .deliver(turn(vec![trigger_event("u1")], Lane::Background, 1))
        .await;
    assert_eq!(response.status, DeliverStatus::Accepted);
    assert!(deliverer.timeout().await <= Duration::from_secs(25));
}
