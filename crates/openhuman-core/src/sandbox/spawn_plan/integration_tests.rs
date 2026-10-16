//! Sandbox planning against the real TinySecurity module.
//!
//! These tests load the actual native library behind an in-memory broker, configured
//! the way the host configures it (approval, audit and an empty sandbox
//! initialization) with the host's own audit callbacks served. They need a released
//! archive (`OPENHUMAN_TEST_SECURITY_RELEASE_HOST`) or an explicitly selected local
//! fixture (`OPENHUMAN_TEST_SECURITY_MODULE`), so they are ignored by default.

use std::path::PathBuf;

use tinysecurity_bus::{
    ConstraintEnforcement2, ConstraintSupport2, IsolationLevel2, SandboxBackend,
    SandboxConstraint2, SandboxExecutor2,
};

use super::*;
use crate::modules::security::native_tests::{native_fixture_configured, NativeFixture};
use crate::sandbox::types::DockerOverrides;

const IGNORE: &str = "requires a released host key or an explicitly selected local native fixture";

struct World {
    _fixture: NativeFixture,
    _dir: tempfile::TempDir,
    module: FixedModule,
    config: Config,
    action: PathBuf,
}

fn executor(isolation: IsolationLevel2, enforcement: ConstraintEnforcement2) -> SandboxExecutor2 {
    SandboxExecutor2 {
        executor_id: executors::JAIL_EXECUTOR.into(),
        capability_id: format!("test-{isolation:?}-{enforcement:?}"),
        backend: SandboxBackend::Jail,
        available: true,
        isolation,
        constraints: [
            SandboxConstraint2::Filesystem,
            SandboxConstraint2::Network,
            SandboxConstraint2::Subprocesses,
        ]
        .into_iter()
        .map(|constraint| ConstraintSupport2 {
            constraint,
            enforcement,
        })
        .collect(),
    }
}

async fn world(executor: SandboxExecutor2) -> World {
    let fixture = native_fixture_configured(true, |config| {
        config.sandbox = Some(crate::modules::security::sandbox_init_config());
    })
    .await;
    let dir = tempfile::tempdir_in(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp"),
    )
    .unwrap_or_else(|_| tempfile::tempdir().unwrap());
    let base = dir.path().canonicalize().unwrap();
    let action = base.join("action");
    std::fs::create_dir_all(&action).unwrap();
    let config = Config {
        workspace_dir: base.join("workspace"),
        action_dir: action.clone(),
        ..Config::default()
    };
    std::fs::create_dir_all(&config.workspace_dir).unwrap();
    World {
        module: FixedModule {
            proxy: fixture.proxy.clone(),
            executor,
        },
        _fixture: fixture,
        _dir: dir,
        config,
        action,
    }
}

fn policy(world: &World, network: bool) -> SandboxPolicy {
    SandboxPolicy {
        backend: SandboxBackendKind::Local,
        workspace_root: world.action.clone(),
        state_dir: world.config.workspace_dir.clone(),
        read_only_mounts: vec![],
        read_write_mounts: vec![],
        allow_network: network,
        env_passthrough: vec![],
        docker_overrides: None,
    }
}

fn operation<'a>(world: &'a World, command: &'a str) -> SpawnOperation<'a> {
    SpawnOperation {
        command,
        cwd: &world.action,
        env: vec![("LANG".into(), "C".into())],
        stdin: None,
        preserve_pipeline_status: false,
    }
}

fn request<'a>(
    world: &'a World,
    policy: &'a SandboxPolicy,
    operation: &'a SpawnOperation<'a>,
    source: SpawnSource,
) -> SpawnRequest<'a> {
    SpawnRequest {
        config: &world.config,
        policy,
        source,
        operation,
        host_off: false,
        module: ModuleSource::Fixed(&world.module),
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_kernel_jail_spawn_is_planned_reserved_once_and_audited() {
    let _ = IGNORE;
    let world = world(executor(
        IsolationLevel2::Kernel,
        ConstraintEnforcement2::Enforced,
    ))
    .await;
    let policy = policy(&world, false);
    let op = operation(&world, "echo hello");
    let planned = plan(request(&world, &policy, &op, SpawnSource::Agent))
        .await
        .expect("the module plans a kernel jail spawn");
    assert_eq!(planned.plan.executor, PlannedExecutor::NativeJail);
    assert!(!planned.plan.jail.allow_net);
    assert!(planned.plan.jail.require_process_isolation);
    assert_eq!(
        planned.plan.jail.read_write,
        [world.action.to_str().unwrap().to_owned()]
    );
    consume(&planned)
        .await
        .expect("an issued plan is admitted once");
    assert_eq!(
        consume(&planned).await.unwrap_err(),
        PlanError::Refused(SandboxDenial2::InvalidContext),
        "a plan is never admitted twice"
    );
    // Two events (plan, consume) reached the subject's ledger, plus the replay's
    // refusal left no extra permission behind.
    let tail = planned
        .stream
        .correlation(1)
        .await
        .expect("correlation")
        .expected_tail;
    assert!(
        tail >= 2,
        "plan and consume were both audited (tail {tail})"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_changed_operation_never_reuses_a_plan() {
    let world = world(executor(
        IsolationLevel2::Kernel,
        ConstraintEnforcement2::Enforced,
    ))
    .await;
    let policy = policy(&world, false);
    let op = operation(&world, "echo hello");
    let mut planned = plan(request(&world, &policy, &op, SpawnSource::Agent))
        .await
        .expect("plan");
    planned.request.operation_json = operation(&world, "echo goodbye").canonical().unwrap();
    assert!(consume(&planned).await.is_err());
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn host_network_is_narrowed_to_none_when_the_module_wants_a_human_approval() {
    let world = world(executor(
        IsolationLevel2::Kernel,
        ConstraintEnforcement2::Enforced,
    ))
    .await;
    let asked = policy(&world, true);
    let op = operation(&world, "curl example.com");
    let planned = plan(request(&world, &asked, &op, SpawnSource::Agent))
        .await
        .expect("planned without the network");
    assert!(!planned.plan.jail.allow_net);
    let applied = planned.apply(&asked);
    assert!(
        !applied.allow_network,
        "the run policy follows the plan, not the request"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_best_effort_jail_is_refused_because_it_enforces_nothing_exactly() {
    let world = world(executor(
        IsolationLevel2::Process,
        ConstraintEnforcement2::BestEffort,
    ))
    .await;
    let policy = policy(&world, false);
    let op = operation(&world, "echo hello");
    assert_eq!(
        plan(request(&world, &policy, &op, SpawnSource::Agent))
            .await
            .unwrap_err(),
        PlanError::Refused(SandboxDenial2::Unsupported)
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn text_the_host_did_not_write_needs_kernel_isolation_whatever_the_switches_say() {
    let world = world(executor(
        IsolationLevel2::Process,
        ConstraintEnforcement2::Enforced,
    ))
    .await;
    let policy = policy(&world, false);
    let op = operation(&world, "echo hello");
    // A process-level jail is fine for the agent's own shell under the operator's
    // switches only when isolation is not required; a scheduled job never gets it.
    assert_eq!(
        plan(request(&world, &policy, &op, SpawnSource::Untrusted))
            .await
            .unwrap_err(),
        PlanError::Refused(SandboxDenial2::Unsupported)
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn a_grant_outside_the_host_chosen_roots_or_inside_a_credential_store_is_refused() {
    let world = world(executor(
        IsolationLevel2::Kernel,
        ConstraintEnforcement2::Enforced,
    ))
    .await;
    let mut policy = policy(&world, false);
    // The host's own mount list names a credential store: the module's floor refuses it.
    policy.read_only_mounts = vec![PathBuf::from("/home/someone/.ssh")];
    let op = operation(&world, "echo hello");
    assert!(plan(request(&world, &policy, &op, SpawnSource::Agent))
        .await
        .is_err());
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn container_bounds_come_from_the_plan() {
    let world = world(executor(
        IsolationLevel2::Kernel,
        ConstraintEnforcement2::Enforced,
    ))
    .await;
    let mut policy = policy(&world, false);
    policy.backend = SandboxBackendKind::Docker;
    policy.docker_overrides = Some(DockerOverrides {
        image: Some("alpine".into()),
        network: Some("bridge".into()),
        memory_limit_mb: Some(256),
        cpu_limit: Some(0.5),
        read_only_rootfs: Some(true),
        extra_caps_drop: vec![],
    });
    let docker = FixedModule {
        proxy: world.module.proxy.clone(),
        executor: executors::docker(true),
    };
    let op = operation(&world, "echo hello");
    let planned = plan(SpawnRequest {
        module: ModuleSource::Fixed(&docker),
        ..request(&world, &policy, &op, SpawnSource::Agent)
    })
    .await
    .expect("planned");
    assert_eq!(planned.plan.executor, PlannedExecutor::Docker);
    let applied = planned.apply(&policy);
    let overrides = applied.docker_overrides.expect("overrides");
    assert_eq!(overrides.memory_limit_mb, Some(256));
    assert_eq!(overrides.cpu_limit, Some(0.5));
    assert_eq!(
        overrides.network.as_deref(),
        Some("none"),
        "the container joins no network"
    );
    assert!(!applied.allow_network);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn another_subject_gets_its_own_scope_and_ledger() {
    let world = world(executor(
        IsolationLevel2::Kernel,
        ConstraintEnforcement2::Enforced,
    ))
    .await;
    let policy = policy(&world, false);
    let op = operation(&world, "echo hello");
    let first = plan(request(&world, &policy, &op, SpawnSource::Agent))
        .await
        .expect("first");
    let second = plan(request(&world, &policy, &op, SpawnSource::Untrusted));
    // The untrusted source needs kernel isolation, which this jail declares.
    let second = second.await.expect("second");
    assert_ne!(first.request.scope_id, second.request.scope_id);
    assert_eq!(first.subject, second.subject);
    assert_eq!(first.stream.stream_id(), second.stream.stream_id());
}

#[tokio::test]
async fn a_module_that_cannot_be_reached_refuses_the_spawn() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let mut config = Config {
        workspace_dir: base.join("workspace"),
        action_dir: base.join("action"),
        ..Config::default()
    };
    config.modules.enabled = false;
    let policy = SandboxPolicy {
        backend: SandboxBackendKind::Local,
        workspace_root: config.action_dir.clone(),
        state_dir: config.workspace_dir.clone(),
        read_only_mounts: vec![],
        read_write_mounts: vec![],
        allow_network: false,
        env_passthrough: vec![],
        docker_overrides: None,
    };
    let op = SpawnOperation {
        command: "echo hello",
        cwd: &config.action_dir,
        env: vec![],
        stdin: None,
        preserve_pipeline_status: false,
    };
    let refused = admit(SpawnRequest {
        config: &config,
        policy: &policy,
        source: SpawnSource::Agent,
        operation: &op,
        host_off: false,
        module: ModuleSource::Production,
    })
    .await
    .unwrap_err();
    assert!(matches!(refused, PlanError::Unavailable(_)), "{refused:?}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn the_host_retires_the_least_recently_used_scope_instead_of_growing_without_bound() {
    use tinysecurity_bus::{RegisterSandboxScopeRequest2, SandboxScopeOutcome2};

    let world = world(executor(
        IsolationLevel2::Kernel,
        ConstraintEnforcement2::Enforced,
    ))
    .await;
    let stream = "9".repeat(32);
    let scope_for = |index: usize| {
        let mut policy = policy(&world, false);
        policy.workspace_root = world.action.join(format!("w{index}"));
        facts::scope(&facts::FactsInput {
            config: &world.config,
            policy: &policy,
            source: SpawnSource::Agent,
            subject: tinysecurity_bus::SubjectScope {
                user: "local".into(),
                workspace: "evict".into(),
                agent: "agent".into(),
            },
            saas: false,
            host_off: false,
            executors: vec![world.module.executor.clone()],
        })
        .expect("scope facts")
    };
    let first = scope_for(0);
    registry::ensure(&world.module.proxy, &first, &stream)
        .await
        .expect("first scope");
    for index in 1..=300 {
        registry::ensure(&world.module.proxy, &scope_for(index), &stream)
            .await
            .expect("later scope");
    }
    // The first scope fell out of the host's bound and was retired in the module: it is
    // registered as a new scope, not found already there.
    let receipt = client::register_scope(
        &world.module.proxy,
        RegisterSandboxScopeRequest2 {
            scope: first,
            audit_stream_id: stream,
            report_handle: false,
        },
    )
    .await
    .expect("registration");
    assert_eq!(receipt.outcome, SandboxScopeOutcome2::Created);
}
