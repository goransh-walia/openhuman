//! Tests for the `inbound` controller schemas.

use super::*;

#[test]
fn every_function_has_a_schema_and_a_controller() {
    let all = all_controller_schemas();
    let names: Vec<&str> = all.iter().map(|schema| schema.function).collect();
    assert_eq!(names, vec!["stats", "dead", "redrive"]);
    assert!(all.iter().all(|schema| schema.namespace == "inbound"));
    let controllers = all_registered_controllers();
    assert_eq!(controllers.len(), 3);
    assert_eq!(schemas("unknown").function, "unknown");
    assert!(schemas("redrive")
        .inputs
        .iter()
        .any(|input| input.name == "id" && input.required));
}

#[tokio::test]
async fn malformed_params_are_refused_before_the_module_is_reached() {
    let mut params = Map::new();
    params.insert("limit".into(), Value::String("ten".into()));
    let refused = handle_dead(DispatchCtx::host_default(), params).await;
    assert!(refused.unwrap_err().contains("invalid params"));
    let refused = handle_redrive(DispatchCtx::host_default(), Map::new()).await;
    assert!(refused.unwrap_err().contains("invalid params"));
}
