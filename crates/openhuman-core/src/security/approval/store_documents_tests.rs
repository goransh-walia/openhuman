use super::*;
use chrono::{Duration, Timelike};
use tinystoragedrivers::{MemoryStorage, Scope, StorageBackend};

fn docs_in(storage: &MemoryStorage, scope: &str) -> Docs {
    Docs::new(&storage.for_scope(&Scope::new(scope).unwrap()).unwrap())
}

fn docs() -> Docs {
    docs_in(&MemoryStorage::new(), "local")
}

fn flow_context() -> ApprovalSourceContext {
    ApprovalSourceContext::Flow {
        flow_id: "flow-1".to_string(),
        run_id: String::new(),
        node_id: None,
    }
}

#[test]
fn a_preauthorization_is_recorded_history() {
    let store = docs();
    store
        .insert_decided(
            "http",
            "pre-authorized",
            "s",
            &flow_context(),
            ApprovalDecision::ApproveAlwaysForFlow,
        )
        .unwrap();
    let audit = store.list_recent_decisions(10).unwrap();
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].decision, ApprovalDecision::ApproveAlwaysForFlow);
}

#[test]
fn recorded_decisions_are_newest_first_and_capped() {
    let store = docs();
    for (tool, at) in [("old", 30), ("new", 10), ("mid", 20)] {
        let decided = rfc3339(Utc::now() - Duration::seconds(at));
        let doc = json!({
            "tool_name": tool,
            "action_summary": "history",
            "args_redacted": "{}",
            "session_id": "s",
            "created_at": decided,
            "pending": false,
            "decided_at": decided,
            "decision": "deny",
        });
        store
            .run(|docs| async move {
                docs.put(APPROVALS, tool, doc, Precondition::Absent)
                    .await
                    .map(|_| ())
            })
            .unwrap();
    }
    let listed = store.list_recent_decisions(2).unwrap();
    let tools: Vec<&str> = listed.iter().map(|e| e.tool_name.as_str()).collect();
    assert_eq!(tools, ["new", "mid"]);
}

#[test]
fn a_pending_legacy_document_is_not_history() {
    let store = docs();
    store
        .run(|docs| async move {
            docs.put(
                APPROVALS,
                "left-pending",
                json!({
                    "tool_name": "shell",
                    "action_summary": "left by an earlier launch",
                    "args_redacted": "{}",
                    "session_id": "s",
                    "created_at": rfc3339(Utc::now()),
                    "pending": true,
                }),
                Precondition::Absent,
            )
            .await
            .map(|_| ())
        })
        .unwrap();
    assert!(store.list_recent_decisions(10).unwrap().is_empty());
}

#[test]
fn flow_trust_grants_list_and_revoke() {
    let store = docs();
    store.insert_flow_trust("flow-1", "shell").unwrap();
    store.insert_flow_trust("flow-1", "shell").unwrap();
    store.insert_flow_trust("flow-1", "http").unwrap();
    store.insert_flow_trust("flow-2", "shell").unwrap();
    assert_eq!(store.list_flow_trust("flow-1").unwrap(), ["http", "shell"]);
    assert_eq!(store.list_all_flow_trust().unwrap().len(), 3);
    assert!(store.is_flow_tool_trusted("flow-1", "shell").unwrap());
    assert!(!store.is_flow_tool_trusted("flow-1", "mail").unwrap());
    assert_eq!(
        store
            .delete_flow_trust("flow-1", Some(&["shell".to_string(), "nope".to_string()]))
            .unwrap(),
        1
    );
    assert_eq!(store.delete_flow_trust("flow-1", None).unwrap(), 1);
    assert!(store.list_flow_trust("flow-1").unwrap().is_empty());
    assert!(store.is_flow_tool_trusted("flow-2", "shell").unwrap());
    assert_ne!(trust_id("a/b", "c"), trust_id("a", "b/c"));
}

#[test]
fn scopes_keep_users_apart() {
    let storage = MemoryStorage::new();
    let alice = docs_in(&storage, "alice");
    let bob = docs_in(&storage, "bob");
    alice.insert_flow_trust("flow-1", "shell").unwrap();
    alice
        .insert_decided(
            "http",
            "pre-authorized",
            "s",
            &flow_context(),
            ApprovalDecision::ApproveAlwaysForFlow,
        )
        .unwrap();
    assert!(!bob.is_flow_tool_trusted("flow-1", "shell").unwrap());
    assert!(bob.list_recent_decisions(10).unwrap().is_empty());
    assert_eq!(alice.list_recent_decisions(10).unwrap().len(), 1);
}

#[test]
fn sub_second_decisions_order_by_time() {
    let store = docs();
    let base = Utc::now().with_nanosecond(0).unwrap();
    for (tool, nanos) in [("first", 100_000_000), ("second", 200_000_000)] {
        let at = rfc3339(base + Duration::nanoseconds(nanos));
        let doc = json!({
            "tool_name": tool,
            "action_summary": "history",
            "args_redacted": "{}",
            "session_id": "s",
            "created_at": at,
            "pending": false,
            "decided_at": at,
            "decision": "approve_once",
        });
        store
            .run(|docs| async move {
                docs.put(APPROVALS, tool, doc, Precondition::Absent)
                    .await
                    .map(|_| ())
            })
            .unwrap();
    }
    let listed = store.list_recent_decisions(10).unwrap();
    assert_eq!(listed[0].tool_name, "second");
    assert_eq!(listed[1].tool_name, "first");
}
