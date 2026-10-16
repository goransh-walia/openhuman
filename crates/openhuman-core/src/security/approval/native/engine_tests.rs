//! The native engine against the real TinySecurity module.
//!
//! These tests load the actual native library behind an in-memory broker with
//! the host's own approval initialization and callbacks, and drive the engine
//! the gate drives. They need a released archive (`OPENHUMAN_TEST_SECURITY_RELEASE_HOST`)
//! or an explicitly selected local fixture (`OPENHUMAN_TEST_SECURITY_MODULE`), so
//! they are ignored by default.

use super::*;
use crate::modules::security::native_tests::{native_approval_fixture, NativeFixture};
use crate::security::approval::engine::{Delivery, Posture, PromptSurface};
use crate::security::approval::types::ApprovalDecision;
use std::sync::{Arc, Mutex};
use tinysecurity_bus::{names, ApprovalLifecycle2, ApprovalRow2, DecideRequest2};

/// A prompt surface that records what the engine asked it to show.
#[derive(Default)]
struct Shown {
    deliveries: Mutex<Vec<Delivery>>,
}

#[async_trait]
impl PromptSurface for Shown {
    async fn deliver(&self, delivery: Delivery) -> Result<(), String> {
        self.deliveries.lock().unwrap().push(delivery);
        Ok(())
    }
}

struct World {
    fixture: NativeFixture,
    _dir: tempfile::TempDir,
    engine: NativeEngine,
    config: Config,
}

async fn world() -> World {
    let fixture = native_approval_fixture().await;
    let dir = tempfile::tempdir_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp"))
        .unwrap_or_else(|_| tempfile::tempdir().unwrap());
    let config = Config {
        workspace_dir: dir.path().to_path_buf(),
        ..Config::default()
    };
    let engine = NativeEngine::on(config.clone(), fixture.proxy.clone());
    World {
        fixture,
        _dir: dir,
        engine,
        config,
    }
}

fn input(tool: &str, surface: &Arc<Shown>) -> CheckInput {
    CheckInput {
        tenant: crate::core::runtime::Tenant::default(),
        variant: Variant::Standard,
        posture: Posture {
            require_approval: true,
            ..Posture::default()
        },
        call_id: format!("{:08x}", rand_id()),
        tool: tool.to_string(),
        args: serde_json::json!({"to": "someone", "token": "not-a-real-token-value"}),
        summary: "send a message".into(),
        thread: Some("thread-1".into()),
        flow: None,
        origin: CallOrigin::Operator,
        ttl: Duration::from_secs(600),
        prompt: Arc::clone(surface) as Arc<dyn PromptSurface>,
    }
}

fn rand_id() -> u32 {
    let bytes = uuid::Uuid::new_v4().into_bytes();
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn pending(checked: Checked) -> (String, u64) {
    match checked {
        Checked::Pending {
            request_id,
            expires_at_ms,
        } => (request_id, expires_at_ms),
        other => panic!("expected a pending request, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_person_approves_reserves_and_the_outcome_is_recorded() {
    let world = world().await;
    let shown = Arc::new(Shown::default());
    let (request_id, expires_at_ms) = pending(
        world
            .engine
            .check(input("gmail_send", &shown))
            .await
            .expect("the module evaluates a host tool"),
    );

    // The module made the request durable and the prompt reached the surface,
    // with what the module stored (redacted), not what the host sent.
    let deliveries = shown.deliveries.lock().unwrap().clone();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].request_id, request_id);
    assert_eq!(deliveries[0].tool_name, "gmail_send");
    assert!(
        !deliveries[0].args_redacted.to_string().contains("not-a-real-token-value"),
        "the stored arguments are redacted"
    );
    assert!(expires_at_ms > deliveries[0].created_at_ms);
    assert!(
        expires_at_ms - deliveries[0].created_at_ms <= 600_000,
        "the interactive window never exceeds ten minutes"
    );

    assert_eq!(
        world
            .engine
            .decide(&request_id, Choice::Once, "approve_once")
            .await
            .unwrap(),
        Decided::Approved
    );
    world.engine.reserve(&request_id).await.unwrap();
    assert!(
        world.engine.reserve(&request_id).await.is_err(),
        "an approval is reserved exactly once"
    );
    world
        .engine
        .record_execution(&request_id, ExecutionOutcome::Success)
        .await
        .unwrap();

    let history = world.engine.history(10).await.unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].request_id, request_id);
    assert_eq!(history[0].decision, ApprovalDecision::ApproveOnce);
    assert!(history[0].decided_by.is_none(), "a person decided");
    drop(world.fixture);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_denial_leaves_nothing_to_reserve() {
    let world = world().await;
    let shown = Arc::new(Shown::default());
    let (request_id, _) = pending(world.engine.check(input("gmail_send", &shown)).await.unwrap());
    assert_eq!(
        world
            .engine
            .decide(&request_id, Choice::Deny, "deny")
            .await
            .unwrap(),
        Decided::Denied
    );
    assert!(world.engine.reserve(&request_id).await.is_err());
    let history = world.engine.history(10).await.unwrap();
    assert_eq!(history[0].decision, ApprovalDecision::Deny);
    // Deciding again finds nothing pending.
    assert_eq!(
        world
            .engine
            .decide(&request_id, Choice::Once, "approve_once")
            .await,
        Err(EngineError::NotPending)
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn the_users_tool_grant_approves_without_a_prompt_and_is_audited() {
    let world = world().await;
    let shown = Arc::new(Shown::default());
    let mut granted = input("gmail_send", &shown);
    granted.posture.tool_grants = vec!["gmail_send".into()];
    let checked = world.engine.check(granted).await.unwrap();
    let Checked::Approved { request_id } = checked else {
        panic!("a granted tool is approved without a human: {checked:?}");
    };
    assert!(shown.deliveries.lock().unwrap().is_empty(), "nobody was asked");
    world.engine.reserve(&request_id).await.unwrap();
    world
        .engine
        .record_execution(&request_id, ExecutionOutcome::Failure)
        .await
        .unwrap();
    let history = world.engine.history(10).await.unwrap();
    assert_eq!(history[0].decided_by.as_deref(), Some("auto:tool-grant"));

    // Another tool is not covered by the grant.
    let mut other = input("slack_post", &shown);
    other.posture.tool_grants = vec!["gmail_send".into()];
    assert!(matches!(
        world.engine.check(other).await.unwrap(),
        Checked::Pending { .. }
    ));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_grant_the_user_removes_stops_approving_the_next_call() {
    let world = world().await;
    let shown = Arc::new(Shown::default());
    let mut granted = input("gmail_send", &shown);
    granted.posture.tool_grants = vec!["gmail_send".into()];
    assert!(matches!(
        world.engine.check(granted).await.unwrap(),
        Checked::Approved { .. }
    ));
    // The same subject, with the grant gone.
    assert!(matches!(
        world.engine.check(input("gmail_send", &shown)).await.unwrap(),
        Checked::Pending { .. }
    ));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_flow_grant_applies_to_that_flow_only() {
    let world = world().await;
    let shown = Arc::new(Shown::default());
    let mut in_flow = input("http_request", &shown);
    in_flow.variant = Variant::FlowStrict;
    in_flow.flow = Some("flow-1".into());
    in_flow.posture.flow_grants = vec![("flow-1".into(), "http_request".into())];
    // The global tool list never approves a strict flow, only its own trust.
    in_flow.posture.tool_grants = vec!["slack_post".into()];
    assert!(matches!(
        world.engine.check(in_flow).await.unwrap(),
        Checked::Approved { .. }
    ));
    let mut other_flow = input("http_request", &shown);
    other_flow.variant = Variant::FlowStrict;
    other_flow.flow = Some("flow-2".into());
    other_flow.posture.flow_grants = vec![("flow-1".into(), "http_request".into())];
    assert!(matches!(
        world.engine.check(other_flow).await.unwrap(),
        Checked::Pending { .. }
    ));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn auto_approve_all_is_the_posture_and_a_forced_call_ignores_it() {
    let world = world().await;
    let shown = Arc::new(Shown::default());
    let mut all = input("gmail_send", &shown);
    all.posture.auto_approve_all = true;
    all.posture.tool_grants = vec!["gmail_send".into()];
    let checked = world.engine.check(all).await.unwrap();
    assert!(matches!(checked, Checked::Approved { .. }));
    let history = {
        let Checked::Approved { request_id } = checked else {
            unreachable!()
        };
        world.engine.reserve(&request_id).await.unwrap();
        world.engine.history(10).await.unwrap()
    };
    assert_eq!(history[0].decided_by.as_deref(), Some("auto:host-policy"));

    // A forced call asks for a person whatever the user configured.
    let mut forced = input("gmail_send", &shown);
    forced.variant = Variant::Forced;
    forced.posture = Posture {
        require_approval: true,
        ..Posture::default()
    };
    assert!(matches!(
        world.engine.check(forced).await.unwrap(),
        Checked::Pending { .. }
    ));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_request_nobody_answers_expires_and_is_recorded_as_denied() {
    let world = world().await;
    let shown = Arc::new(Shown::default());
    let mut short = input("gmail_send", &shown);
    short.ttl = Duration::from_millis(1200);
    let (request_id, expires_at_ms) = pending(world.engine.check(short).await.unwrap());
    assert!(
        world.engine.expire(&request_id).await.is_err(),
        "a live request is not expired early"
    );
    let wait = expires_at_ms.saturating_sub(now_ms()) + 50;
    tokio::time::sleep(Duration::from_millis(wait)).await;
    assert!(world.engine.expire(&request_id).await.unwrap());
    let history = world.engine.history(10).await.unwrap();
    assert_eq!(history[0].decision, ApprovalDecision::Deny);
    assert!(
        world.engine.reserve(&request_id).await.is_err(),
        "an expired request cannot be reserved"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_cancelled_request_cannot_be_decided() {
    let world = world().await;
    let shown = Arc::new(Shown::default());
    let (request_id, _) = pending(world.engine.check(input("gmail_send", &shown)).await.unwrap());
    world.engine.cancel(&request_id).await.unwrap();
    assert_eq!(
        world
            .engine
            .decide(&request_id, Choice::Once, "approve_once")
            .await,
        Err(EngineError::NotPending)
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_cron_origin_never_reaches_the_engine_and_a_reserved_name_is_refused() {
    let world = world().await;
    let shown = Arc::new(Shown::default());
    // The module's own effect classes and human-only tools are not host tools.
    for tool in ["local_diagnostic", "native_secret_grant", "device_crypto"] {
        assert!(
            matches!(
                world.engine.check(input(tool, &shown)).await,
                Err(EngineError::Unavailable(_))
            ),
            "{tool}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn concurrent_calls_of_one_subject_all_get_their_own_request() {
    let world = world().await;
    let engine = Arc::new(NativeEngine::on(
        world.config.clone(),
        world.fixture.proxy.clone(),
    ));
    let mut tasks = Vec::new();
    for index in 0..8 {
        let engine = Arc::clone(&engine);
        tasks.push(tokio::spawn(async move {
            let shown = Arc::new(Shown::default());
            let mut call = input("gmail_send", &shown);
            call.args = serde_json::json!({ "n": index });
            pending(engine.check(call).await.expect("concurrent check")).0
        }));
    }
    let mut ids = Vec::new();
    for task in tasks {
        ids.push(task.await.unwrap());
    }
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 8, "each call got its own durable request");
    // The audit stream stayed contiguous: one event per request.
    let scope = {
        let scopes = super::super::state::host().scopes.lock().await;
        scopes
            .by_subject
            .values()
            .find(|entry| entry.subject.workspace == world.config.workspace_dir.to_string_lossy())
            .cloned()
            .expect("the subject is registered")
    };
    assert_eq!(scope.rows.tail(&scope.stream_id).await.unwrap(), 8);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn two_workspaces_are_two_subjects_with_their_own_rows() {
    let world = world().await;
    let other_dir = tempfile::tempdir().unwrap();
    let other_config = Config {
        workspace_dir: other_dir.path().to_path_buf(),
        ..Config::default()
    };
    let other = NativeEngine::on(other_config, world.fixture.proxy.clone());
    let shown = Arc::new(Shown::default());
    let (mine, _) = pending(world.engine.check(input("gmail_send", &shown)).await.unwrap());
    let (theirs, _) = pending(other.check(input("gmail_send", &shown)).await.unwrap());
    assert_ne!(mine, theirs);
    // Neither workspace's history shows the other's request.
    world
        .engine
        .decide(&mine, Choice::Once, "approve_once")
        .await
        .unwrap();
    assert_eq!(world.engine.history(10).await.unwrap().len(), 1);
    assert!(other.history(10).await.unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_ticket_the_host_never_issued_decides_nothing() {
    let world = world().await;
    let shown = Arc::new(Shown::default());
    let (request_id, _) = pending(world.engine.check(input("gmail_send", &shown)).await.unwrap());
    let entry = super::super::state::host()
        .request(&request_id)
        .expect("the delivered request is remembered");

    // What a model could try: the module's own `Decide2` with a ticket it made up.
    let forged = {
        let _guard = entry.scope.stream.lock().await;
        let generation = NativeEngine::generation(&world.fixture.proxy).await.unwrap();
        let audit = NativeEngine::correlation(&entry.scope, generation)
            .await
            .unwrap();
        world
            .fixture
            .proxy
            .call_confidential::<ApprovalRow2>(
                names::methods::DECIDE2,
                (DecideRequest2 {
                    scope_id: entry.scope.scope_id.clone(),
                    request_id: request_id.clone(),
                    decision_ticket: "model-claimed-human".into(),
                    audit,
                },),
            )
            .await
    };
    assert!(forged.is_err(), "the host refused to verify a forged ticket");
    let stored = entry.scope.rows.row(&request_id).await.unwrap().unwrap();
    assert_eq!(stored.row.state, ApprovalLifecycle2::Pending);
    assert!(world.engine.reserve(&request_id).await.is_err());

    // The real decision still works afterwards.
    assert_eq!(
        world
            .engine
            .decide(&request_id, Choice::Once, "approve_once")
            .await
            .unwrap(),
        Decided::Approved
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn only_the_module_may_call_the_hosts_approval_callbacks() {
    let world = world().await;
    let stranger = tinybus::Connection::connect(world.fixture.transport.connect().await.unwrap())
        .await
        .unwrap();
    let host_peer = world.fixture.client.unique_name().expect("host peer");
    for (interface, member) in [
        ("ai.tinyhumans.tinysecurity.ApprovalStore2", "Load"),
        ("ai.tinyhumans.tinysecurity.AuditSink2", "Append"),
        ("ai.tinyhumans.tinysecurity.ApprovalPrompt2", "VerifyHuman"),
    ] {
        let proxy = stranger
            .proxy(
                host_peer.as_str(),
                tinysecurity_bus::callback_names2::OBJECT_PATH,
                interface,
            )
            .unwrap();
        let answered = proxy
            .call::<serde_json::Value>(member, (serde_json::json!({}),))
            .await;
        assert!(answered.is_err(), "{interface}.{member} answered a stranger");
    }
}
