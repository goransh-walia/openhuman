use super::*;
use crate::modules::security::native_tests::native_approval_fixture;
use tinysecurity_bus::{AuditEventReceipt2, AuditHealth2, AuditSinkResponse2};

fn subject(agent: &str) -> SubjectScope {
    SubjectScope {
        user: "local".into(),
        workspace: "/work".into(),
        agent: agent.into(),
    }
}

fn correlation() -> AuditCorrelation2 {
    AuditCorrelation2 {
        event_id: "a".repeat(32),
        expected_tail: 4,
        generation: 1,
    }
}

fn answer(committed: bool, new_tail: u64, event_id: &str) -> AuditResponse2 {
    AuditResponse2 {
        permitted: true,
        committed,
        health: if committed {
            AuditHealth2::Healthy
        } else {
            AuditHealth2::Degraded
        },
        receipt: committed.then(|| AuditSinkResponse2 {
            scope: subject("agent"),
            stream_id: "b".repeat(32),
            events: vec![AuditEventReceipt2 {
                sequence: new_tail,
                event_id: event_id.into(),
            }],
            new_tail,
        }),
    }
}

#[test]
fn only_a_receipt_for_exactly_this_event_counts_as_recorded() {
    let audit = correlation();
    assert!(committed(&answer(true, 5, &audit.event_id), &audit).is_ok());
    // Not committed (the sink was unreachable and audit is optional): a failure.
    assert!(committed(&answer(false, 0, ""), &audit).is_err());
    // Another event, or another position, is not this row.
    assert!(committed(&answer(true, 5, &"c".repeat(32)), &audit).is_err());
    assert!(committed(&answer(true, 7, &audit.event_id), &audit).is_err());
    // A permitted answer without a receipt never claims a commit.
    let mut bare = answer(true, 5, &audit.event_id);
    bare.receipt = None;
    assert!(committed(&bare, &audit).is_err());
}

#[test]
fn a_host_row_maps_to_the_module_kind_and_carries_the_streams_identity() {
    let row = |kind| HostAuditEvent {
        kind,
        attribution: "tool:shell".into(),
        arguments: serde_json::json!({"command": "ls"}),
    };
    let stream_id = "7".repeat(32);
    let execution = request(
        &stream_id,
        subject("agent"),
        row(HostAuditKind::Execution),
        correlation(),
    )
    .unwrap();
    assert_eq!(execution.kind, AuditKind2::Execution);
    assert_eq!(execution.stream_id, stream_id);
    assert_eq!(execution.subject, subject("agent"));
    assert_eq!(execution.audit, correlation());
    assert_eq!(execution.arguments_json, r#"{"command":"ls"}"#);
    assert!(execution.validate().is_ok());
    let decision = request(
        &stream_id,
        subject("agent"),
        row(HostAuditKind::PolicyDecision),
        correlation(),
    )
    .unwrap();
    assert_eq!(decision.kind, AuditKind2::PolicyDecision);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn shell_rows_extend_the_subjects_one_ordered_stream_and_are_redacted() {
    let fixture = native_approval_fixture().await;
    let config = Config {
        workspace_dir: fixture.acting.path().join("workspace"),
        ..Config::default()
    };
    std::fs::create_dir_all(&config.workspace_dir).unwrap();
    let who = subject("recorded-agent");
    let token = ["sk", "-", &"0123456789abcdef".repeat(2)].concat();
    for index in 0..2 {
        let event = HostAuditEvent::command_execution(
            "tool:shell",
            &format!("echo {index} {token}"),
            false,
            true,
            true,
            5,
        );
        record_with(&fixture.proxy, &config, who.clone(), event)
            .await
            .expect("the module records a host row on the subject's stream");
    }
    let stream = AuditStream::open(&config, who).await.unwrap();
    let next = stream.correlation(1).await.unwrap();
    assert_eq!(
        next.expected_tail, 2,
        "both rows extend one contiguous stream"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_row_the_module_cannot_commit_fails_instead_of_vanishing() {
    let fixture = native_approval_fixture().await;
    let config = Config {
        workspace_dir: fixture.acting.path().join("workspace"),
        ..Config::default()
    };
    std::fs::create_dir_all(&config.workspace_dir).unwrap();
    // A subject identity the module refuses (an empty agent) is an Err, not a
    // silent success and not a write to another sink.
    let event = HostAuditEvent::command_execution("tool:shell", "ls", false, true, true, 1);
    let refused = record_with(&fixture.proxy, &config, subject(""), event).await;
    assert!(refused.is_err());
}
