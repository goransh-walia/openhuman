use super::*;
use tinysecurity_bus::{PathGrant, ResourceLimits2, SubjectScope};

fn artifact(backend: SandboxBackend) -> SandboxArtifact2 {
    SandboxArtifact2 {
        minimum_isolation: IsolationLevel2::Kernel,
        plan_id: "plan".into(),
        approval_reservation_id: None,
        fingerprint: "fp".into(),
        scope_id: "scope".into(),
        subject: SubjectScope {
            user: "u".into(),
            workspace: "w".into(),
            agent: "a".into(),
        },
        source_id: "source".into(),
        provenance: "prov".into(),
        call_id: "call".into(),
        generation: 1,
        expires_at_ms: 1,
        executor_id: "exec".into(),
        capability_id: "cap".into(),
        backend,
        grants: vec![
            PathGrant {
                path: "/work".into(),
                write: true,
            },
            PathGrant {
                path: "/toolchain".into(),
                write: false,
            },
        ],
        network: PlannedNetwork::None,
        proxy_id: None,
        limits: ResourceLimits2::default(),
        constraints: vec![SandboxConstraint2::Filesystem, SandboxConstraint2::Network],
    }
}

#[test]
fn a_jail_plan_maps_grants_network_and_isolation_floor() {
    let plan = BoxPlan::from_artifact(&artifact(SandboxBackend::Jail), None).unwrap();
    assert_eq!(plan.executor, PlannedExecutor::NativeJail);
    assert_eq!(plan.jail.read_write, ["/work"]);
    assert_eq!(plan.jail.read_only, ["/toolchain"]);
    assert!(!plan.jail.allow_net);
    assert!(plan.jail.allow_subprocess);
    assert_eq!(plan.jail.enforcement, JailEnforcement::Enforced);
    assert!(plan.jail.require_process_isolation);
    assert_eq!(plan.network, NetworkPolicy::Denied);
    assert_eq!(plan.resources, ResourceLimits::default());
}

#[test]
fn planned_bounds_override_defaults_and_absent_bounds_keep_them() {
    let mut planned = artifact(SandboxBackend::Docker);
    planned.limits = ResourceLimits2 {
        cpu_time_ms: None,
        cpu_millicores: Some(500),
        memory_bytes: Some(64 << 20),
        pids_max: None,
        disk_bytes: None,
    };
    planned.constraints.push(SandboxConstraint2::Subprocesses);
    let plan = BoxPlan::from_artifact(&planned, None).unwrap();
    let defaults = ResourceLimits::default();
    assert_eq!(plan.executor, PlannedExecutor::Docker);
    assert_eq!(plan.resources.cpu_millis, 500);
    assert_eq!(plan.resources.memory_bytes, 64 << 20);
    assert_eq!(plan.resources.pids_max, defaults.pids_max);
    assert_eq!(plan.resources.disk_bytes, defaults.disk_bytes);
    assert!(!plan.jail.allow_subprocess);
}

#[test]
fn host_networking_opens_the_box_only_when_planned() {
    let mut planned = artifact(SandboxBackend::Jail);
    planned.network = PlannedNetwork::Host;
    let plan = BoxPlan::from_artifact(&planned, None).unwrap();
    assert_eq!(plan.network, NetworkPolicy::Open);
    assert!(plan.jail.allow_net);
}

#[test]
fn a_process_level_plan_carries_the_credential_store_floor() {
    let home = std::path::Path::new("/home/someone");
    let mut planned = artifact(SandboxBackend::Jail);
    planned.minimum_isolation = IsolationLevel2::Process;
    let plan = BoxPlan::from_artifact(&planned, Some(home)).unwrap();
    assert!(!plan.jail.require_process_isolation);
    assert!(plan
        .jail
        .deny_read
        .contains(&"/home/someone/.ssh".to_owned()));

    // A kernel-isolated jail is allow-list based and names nothing.
    planned.minimum_isolation = IsolationLevel2::Kernel;
    let plan = BoxPlan::from_artifact(&planned, Some(home)).unwrap();
    assert!(plan.jail.deny_read.is_empty());
}

#[test]
fn plans_tinybox_cannot_enforce_exactly_are_refused() {
    assert_eq!(
        BoxPlan::from_artifact(&artifact(SandboxBackend::Noop), None),
        Err(PlanMappingError::NoopBackend)
    );
    for backend in [SandboxBackend::Namespace, SandboxBackend::MicroVm] {
        assert_eq!(
            BoxPlan::from_artifact(&artifact(backend), None),
            Err(PlanMappingError::ExecutorNotWired(backend))
        );
    }

    let mut proxied = artifact(SandboxBackend::Jail);
    proxied.network = PlannedNetwork::EgressProxy;
    assert_eq!(
        BoxPlan::from_artifact(&proxied, None),
        Err(PlanMappingError::EgressProxyUnavailable)
    );

    let mut cpu_time = artifact(SandboxBackend::Jail);
    cpu_time.limits.cpu_time_ms = Some(1000);
    assert_eq!(
        BoxPlan::from_artifact(&cpu_time, None),
        Err(PlanMappingError::CpuTimeUnenforceable)
    );

    let mut zero = artifact(SandboxBackend::Jail);
    zero.limits.memory_bytes = Some(0);
    assert_eq!(
        BoxPlan::from_artifact(&zero, None),
        Err(PlanMappingError::ZeroLimit)
    );

    let mut empty = artifact(SandboxBackend::Jail);
    empty.grants[0].path.clear();
    assert_eq!(
        BoxPlan::from_artifact(&empty, None),
        Err(PlanMappingError::InvalidGrant)
    );
}
