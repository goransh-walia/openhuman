use super::*;
use crate::sandbox::types::DockerOverrides;
use std::path::PathBuf;

fn policy(backend: SandboxBackendKind) -> SandboxPolicy {
    SandboxPolicy {
        backend,
        workspace_root: PathBuf::from("/work/action"),
        state_dir: PathBuf::from("/state"),
        read_only_mounts: vec![PathBuf::from("/opt")],
        read_write_mounts: vec![PathBuf::from("/home/u/.cargo/registry")],
        allow_network: false,
        env_passthrough: vec![],
        docker_overrides: None,
    }
}

fn with_overrides(cpu: Option<f64>, memory_mb: Option<u64>) -> SandboxPolicy {
    SandboxPolicy {
        docker_overrides: Some(DockerOverrides {
            cpu_limit: cpu,
            memory_limit_mb: memory_mb,
            ..DockerOverrides::default()
        }),
        ..policy(SandboxBackendKind::Docker)
    }
}

#[test]
fn bounds_come_from_the_docker_overrides_and_never_loosen() {
    assert_eq!(limits(&policy(SandboxBackendKind::Local)), ResourceLimits2::default());
    let bounded = limits(&with_overrides(Some(1.5), Some(512)));
    assert_eq!(bounded.cpu_millicores, Some(1500));
    assert_eq!(bounded.memory_bytes, Some(512 * 1024 * 1024));
    assert_eq!(bounded.cpu_time_ms, None);
    // A fraction rounds up to the next millicore rather than down to a looser bound.
    assert_eq!(limits(&with_overrides(Some(0.0004), None)).cpu_millicores, Some(1));
    // Absent, zero and non-finite bounds are not bounds.
    for cpu in [None, Some(0.0), Some(-1.0), Some(f64::NAN), Some(f64::INFINITY)] {
        assert_eq!(limits(&with_overrides(cpu, None)).cpu_millicores, None, "{cpu:?}");
    }
    assert_eq!(limits(&with_overrides(None, Some(0))).memory_bytes, None);
    assert_eq!(limits(&with_overrides(None, Some(u64::MAX))).memory_bytes, None);
}

#[test]
fn a_local_jail_asks_for_every_mount_and_a_container_for_none_it_cannot_write() {
    let local = grants(&policy(SandboxBackendKind::Local)).expect("grants");
    let described: Vec<_> = local.iter().map(|g| (g.path.as_str(), g.write)).collect();
    assert_eq!(
        described,
        [
            ("/home/u/.cargo/registry", true),
            ("/opt", false),
            ("/work/action", true),
        ]
    );
    let docker = grants(&policy(SandboxBackendKind::Docker)).expect("grants");
    let described: Vec<_> = docker.iter().map(|g| (g.path.as_str(), g.write)).collect();
    assert_eq!(described, [("/opt", false), ("/work/action", true)]);
}

#[test]
fn the_plan_must_use_the_executor_the_policy_selected() {
    let plan = |executor| BoxPlan {
        executor,
        jail: crate::modules::tinybox::contract::NativeJailConfig {
            read_only: vec![],
            read_write: vec![],
            allow_net: false,
            allow_subprocess: true,
            enforcement: crate::modules::tinybox::contract::JailEnforcement::Enforced,
            require_process_isolation: true,
            deny_read: vec![],
        },
        network: crate::modules::tinybox::contract::NetworkPolicy::Denied,
        resources: crate::modules::tinybox::contract::ResourceLimits::default(),
    };
    let local = policy(SandboxBackendKind::Local);
    let docker = policy(SandboxBackendKind::Docker);
    assert!(ensure_selected(&plan(PlannedExecutor::NativeJail), &local).is_ok());
    assert!(ensure_selected(&plan(PlannedExecutor::Docker), &docker).is_ok());
    assert!(ensure_selected(&plan(PlannedExecutor::Docker), &local).is_err());
    assert!(ensure_selected(&plan(PlannedExecutor::NativeJail), &docker).is_err());
    assert!(ensure_selected(
        &plan(PlannedExecutor::NativeJail),
        &policy(SandboxBackendKind::None)
    )
    .is_err());
}

#[test]
fn refusals_name_the_denial_and_never_the_operation() {
    let refused = PlanError::Refused(SandboxDenial2::Approval);
    assert!(refused.to_string().contains("Approval"));
    let unavailable = PlanError::Unavailable("no module".into());
    assert!(unavailable.to_string().contains("no module"));
    assert_ne!(refused, unavailable);
}
