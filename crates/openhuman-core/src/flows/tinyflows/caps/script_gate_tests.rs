use std::collections::BTreeMap;
use std::time::Duration;
use tinyflows::caps::host::{require_script_spawner, run_script, ScriptLanguage, ScriptRequest};

/// Building flow capabilities leaves tinyflows' script runner unable to spawn
/// an unjailed interpreter: with no spawner installed the run is refused, not
/// started directly.
#[tokio::test]
async fn building_capabilities_refuses_an_unjailed_script_run() {
    require_script_spawner(false);
    let config = std::sync::Arc::new(crate::config::Config::default());
    let _ = super::super::build_capabilities(config, "flow:test", None);

    let env = BTreeMap::new();
    let input = serde_json::json!({});
    let error = run_script(ScriptRequest::plain(
        ScriptLanguage::Python,
        "print(1)",
        &input,
        Duration::from_secs(5),
        &env,
    ))
    .await
    .expect_err("no spawner is installed, so the run must be refused");
    assert!(error.to_string().contains("fail closed"), "{error}");
}
