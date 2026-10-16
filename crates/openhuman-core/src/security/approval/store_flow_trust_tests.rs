use super::*;

#[test]
fn flow_tool_trust_round_trips() {
    let (config, _dir) = test_config();
    assert!(!is_flow_tool_trusted(&config, "flow-1", "composio").unwrap());

    insert_flow_trust(&config, "flow-1", "composio").unwrap();

    assert!(is_flow_tool_trusted(&config, "flow-1", "composio").unwrap());
    // A different tool on the same flow, or the same tool on a different
    // flow, must not be trusted by the one grant.
    assert!(!is_flow_tool_trusted(&config, "flow-1", "pushover").unwrap());
    assert!(!is_flow_tool_trusted(&config, "flow-2", "composio").unwrap());
}

#[test]
fn insert_flow_trust_is_idempotent() {
    let (config, _dir) = test_config();
    insert_flow_trust(&config, "flow-1", "composio").unwrap();
    // A second grant of the same pair must not error (INSERT OR IGNORE).
    insert_flow_trust(&config, "flow-1", "composio").unwrap();
    assert!(is_flow_tool_trusted(&config, "flow-1", "composio").unwrap());
}

#[test]
fn list_flow_trust_returns_sorted_grants_scoped_to_flow() {
    let (config, _dir) = test_config();
    assert!(list_flow_trust(&config, "flow-1").unwrap().is_empty());

    insert_flow_trust(&config, "flow-1", "zeta_tool").unwrap();
    insert_flow_trust(&config, "flow-1", "alpha_tool").unwrap();
    insert_flow_trust(&config, "flow-2", "other_tool").unwrap();

    assert_eq!(
        list_flow_trust(&config, "flow-1").unwrap(),
        vec!["alpha_tool".to_string(), "zeta_tool".to_string()],
    );
}

#[test]
fn delete_flow_trust_named_removes_only_named_grants() {
    let (config, _dir) = test_config();
    insert_flow_trust(&config, "flow-1", "slack_post").unwrap();
    insert_flow_trust(&config, "flow-1", "gmail_send").unwrap();

    let removed = delete_flow_trust(&config, "flow-1", Some(&["slack_post".to_string()])).unwrap();
    assert_eq!(removed, 1);
    assert!(!is_flow_tool_trusted(&config, "flow-1", "slack_post").unwrap());
    assert!(is_flow_tool_trusted(&config, "flow-1", "gmail_send").unwrap());
    // Revoking a never-granted name is a no-op, not an error.
    let removed = delete_flow_trust(&config, "flow-1", Some(&["missing".to_string()])).unwrap();
    assert_eq!(removed, 0);
}

#[test]
fn delete_flow_trust_all_purges_only_that_flow() {
    let (config, _dir) = test_config();
    insert_flow_trust(&config, "flow-1", "slack_post").unwrap();
    insert_flow_trust(&config, "flow-1", "gmail_send").unwrap();
    insert_flow_trust(&config, "flow-2", "slack_post").unwrap();

    let removed = delete_flow_trust(&config, "flow-1", None).unwrap();
    assert_eq!(removed, 2);
    assert!(list_flow_trust(&config, "flow-1").unwrap().is_empty());
    assert!(is_flow_tool_trusted(&config, "flow-2", "slack_post").unwrap());
}

#[test]
fn list_all_flow_trust_returns_every_flows_grants() {
    let (config, _dir) = test_config();
    assert!(list_all_flow_trust(&config).unwrap().is_empty());
    insert_flow_trust(&config, "flow-1", "gmail_send").unwrap();
    insert_flow_trust(&config, "flow-2", "slack_post").unwrap();
    let mut all = list_all_flow_trust(&config).unwrap();
    all.sort();
    assert_eq!(
        all,
        [
            ("flow-1".to_string(), "gmail_send".to_string()),
            ("flow-2".to_string(), "slack_post".to_string())
        ]
    );
}

#[test]
fn record_flow_preauthorization_lands_in_the_recorded_history() {
    let (config, _dir) = test_config();
    record_flow_preauthorization(&config, "flow-1", "slack_post", "session-test").unwrap();

    // Born-decided: visible in the durable audit trail with the flow decision.
    let decisions = list_recorded_decisions(&config, 10).unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].tool_name, "slack_post");
    assert_eq!(
        decisions[0].decision,
        ApprovalDecision::ApproveAlwaysForFlow
    );
}
