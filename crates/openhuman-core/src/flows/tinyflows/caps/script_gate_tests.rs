use super::*;
use std::collections::BTreeMap;
use std::time::Duration;
use tinyflows::caps::host::{run_script, ScriptLanguage, ScriptRequest};

#[tokio::test]
async fn an_unjailed_script_run_is_refused_once_the_gate_is_set() {
    require_jailed_scripts();
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

#[test]
fn building_capabilities_sets_the_gate() {
    let config = std::sync::Arc::new(crate::config::Config::default());
    let _ = super::super::build_capabilities(config, "flow:test", None);
    let state = tinyflows::caps::host::script_spawner_required();
    assert!(state);
}
