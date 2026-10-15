use super::*;
use tinysecurity_bus::{PathAccess, PathPolicy, PathTrustedRoot, PathValidationResult};

/// The production client (`register_path_policy` -> `proxy` -> `ensure_loaded`
/// -> `module_config`) against the published archive loaded through the real
/// module runtime. Unlike the fixture-backed tests it takes the trusted host
/// from `module_config` itself, so it fails with `RegisterPathPolicy failed`
/// if that configuration ever stops naming the host's bus connection.
///
/// Process-global state (module runtime, resolution table, client state), so
/// run it with the rest of `modules::security` only through the native lane.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key (downloads the published archive)"]
async fn production_client_registers_and_checks_a_path_policy_on_the_released_archive() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
    std::fs::create_dir_all(&root).unwrap();
    let dir = tempfile::tempdir_in(&root).unwrap();
    let base = dir.path().canonicalize().unwrap();
    let action = base.join("action");
    std::fs::create_dir(&action).unwrap();
    std::fs::write(action.join("readable.txt"), "fixture").unwrap();
    let mut config = Config::default();
    config.modules.enabled = true;
    config.modules.allow_download = true;
    config.modules.install_dir = Some(base.join("modules").to_string_lossy().into_owned());
    config.action_dir = action.clone();
    config.workspace_dir = base.join("state");

    let policy = PathPolicy {
        enabled: true,
        workspace_only: true,
        workspace_dir: base.join("state"),
        action_dir: action.clone(),
        home_dir: None,
        forbidden_paths: vec![],
        trusted_roots: vec![PathTrustedRoot {
            path: action.clone(),
            access: PathAccess::ReadWrite,
        }],
        reserved_paths: vec![],
        reserved_prefixes: vec![],
        readonly_paths: vec![],
        reserved_names: vec![],
    };
    let id = register_path_policy(&config, policy)
        .await
        .expect("production RegisterPathPolicy must be accepted from the trusted host");
    assert!(
        module_config(&config).trusted_host.is_some(),
        "module_config must name the host's bus connection once the runtime is up"
    );
    assert_eq!(
        validate_path(&config, id.clone(), "readable.txt".into())
            .await
            .unwrap(),
        PathValidationResult::Allowed {
            resolved: action.join("readable.txt")
        }
    );
    assert!(matches!(
        validate_path(&config, id, "../outside".into())
            .await
            .unwrap(),
        PathValidationResult::Denied { .. }
    ));
}
