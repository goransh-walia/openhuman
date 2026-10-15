use super::*;

fn config_in(root: &Path) -> Config {
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let mut config = Config {
        workspace_dir: workspace.clone(),
        action_dir: workspace,
        config_path: root.join("config.toml"),
        ..Config::default()
    };
    config.modules.enabled = false;
    config
}

fn shell<'a>(config: &'a Config, command: &'a str) -> UntrustedShell<'a> {
    UntrustedShell {
        config,
        command,
        cwd: &config.action_dir,
        state_dir: &config.workspace_dir,
        timeout: Duration::from_secs(10),
        preserve_pipeline_status: false,
    }
}

/// With no TinyBox module the jail cannot be reached, and an untrusted command
/// must be refused rather than started on the host.
#[tokio::test]
async fn an_untrusted_command_never_runs_when_the_jail_is_unreachable() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_in(tmp.path());
    let marker = tmp.path().join("ran");
    let command = format!("touch '{}'", marker.display());

    let result = run(shell(&config, &command)).await;

    assert!(result.is_err(), "the jail is unreachable: {result:?}");
    assert!(!marker.exists(), "a refused spawn must never run the command");
}

/// The operator's own opt-out is the only unjailed path in a single-user host.
#[tokio::test]
async fn the_operator_opt_out_runs_the_command_unjailed() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = config_in(tmp.path());
    config.sandbox.enabled = Some(false);

    let output = run(shell(&config, "echo opted-out")).await.unwrap();

    assert!(output.success());
    assert!(output.stdout.contains("opted-out"), "{}", output.stdout);
}

const SAAS_CHILD: &str = "OPENHUMAN_UNTRUSTED_SAAS_TEST_CHILD";

/// SaaS never honours an opt-out: with the config and the host switch both
/// turning the sandbox off, the command still must not start on the host.
/// Deployment mode is process-owned and irreversible, so the SaaS half runs in
/// a fresh libtest process.
#[tokio::test]
async fn saas_refuses_an_untrusted_command_even_when_the_operator_opted_out() {
    let module = module_path!().split_once("::").expect("crate module").1;
    let test = format!("{module}::saas_refuses_an_untrusted_command_even_when_the_operator_opted_out");
    if std::env::var(SAAS_CHILD).as_deref() == Ok(test.as_str()) {
        crate::core::runtime::mode::lock_mode(crate::core::runtime::Mode::Saas).unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let mut config = config_in(tmp.path());
        config.sandbox.enabled = Some(false);
        let marker = tmp.path().join("ran");
        let command = format!("touch '{}'", marker.display());

        let result = run(shell(&config, &command)).await;

        assert!(result.is_err(), "SaaS must not run it unjailed: {result:?}");
        assert!(!marker.exists(), "the command ran on the host under SaaS");
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", &test, "--nocapture", "--test-threads=1"])
        .env(SAAS_CHILD, &test)
        .env(super::super::ops::SANDBOX_OFF_ENV, "off")
        .output()
        .expect("run isolated SaaS test");
    assert!(
        output.status.success() && String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "isolated test {test} failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Drives the real gate against the released TinyBox archive: the command runs
/// inside the native jail, so it can write its workspace and cannot write a
/// sibling directory. Run with
/// `cargo test -p openhuman --lib untrusted_command_is_confined -- --ignored`
/// (downloads the pinned archive into a scratch install under `target/tmp`).
#[cfg(all(feature = "modules", target_os = "linux"))]
#[tokio::test]
#[ignore = "requires the released TinyBox archive and a supported Linux jail"]
async fn untrusted_command_is_confined_by_the_released_native_jail() {
    let _environment_lock = crate::config::TEST_ENV_LOCK.lock().await;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
    std::fs::create_dir_all(&root).unwrap();
    let tmp = tempfile::tempdir_in(&root).unwrap();
    let base = tmp.path().canonicalize().unwrap();
    let workspace = base.join("workspace");
    let outside = base.join("outside");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    let mut config = Config {
        workspace_dir: workspace.clone(),
        action_dir: workspace.clone(),
        config_path: base.join("config.toml"),
        ..Config::default()
    };
    config.modules.enabled = true;
    config.modules.allow_download = true;
    config.modules.install_dir = Some(base.join("modules").display().to_string());
    let state = base.join("state");

    let inside = run(UntrustedShell {
        config: &config,
        command: "echo jailed > inside.txt && cat inside.txt",
        cwd: &workspace,
        state_dir: &state,
        timeout: Duration::from_secs(30),
        preserve_pipeline_status: false,
    })
    .await
    .expect("the released jail runs a workspace write");
    assert!(inside.success(), "{inside:?}");
    assert!(inside.stdout.contains("jailed"), "{}", inside.stdout);

    let escape = outside.join("escaped.txt");
    let command = format!("echo escaped > '{}'", escape.display());
    let outcome = run(UntrustedShell {
        config: &config,
        command: &command,
        cwd: &workspace,
        state_dir: &state,
        timeout: Duration::from_secs(30),
        preserve_pipeline_status: false,
    })
    .await;
    assert!(
        !escape.exists(),
        "the jail let an untrusted command write outside its workspace: {outcome:?}"
    );
}
