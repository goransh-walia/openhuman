use super::*;
use tinystoragedrivers::{MemoryStorage, Scope, StorageBackend};

fn subject(agent: &str) -> SubjectScope {
    SubjectScope {
        user: "local".into(),
        workspace: "/work".into(),
        agent: agent.into(),
    }
}

fn entry(agent: &str) -> Arc<ScopeEntry> {
    let storage = MemoryStorage::new();
    let scoped = storage.for_scope(&Scope::new("local").unwrap()).unwrap();
    Arc::new(ScopeEntry::new(
        subject(agent),
        "5".repeat(32),
        Rows::over(Arc::clone(scoped.documents())),
    ))
}

fn posture(tools: &[&str], flows: &[(&str, &str)]) -> Posture {
    Posture {
        require_approval: true,
        auto_approve_all: false,
        tool_grants: tools.iter().map(|t| (*t).to_owned()).collect(),
        flow_grants: flows
            .iter()
            .map(|(flow, tool)| ((*flow).to_owned(), (*tool).to_owned()))
            .collect(),
    }
}

#[test]
fn grants_are_sorted_deduplicated_and_exact() {
    let granted = grants(&posture(
        &["slack_post", "gmail_send", "slack_post"],
        &[("flow-b", "http"), ("flow-a", "http"), ("flow-a", "http")],
    ));
    assert_eq!(granted.tools, ["gmail_send", "slack_post"]);
    let flows: Vec<_> = granted
        .flows
        .iter()
        .map(|g| (g.flow_id.as_str(), g.tool.as_str()))
        .collect();
    assert_eq!(flows, [("flow-a", "http"), ("flow-b", "http")]);
    assert!(granted.is_valid());
}

#[test]
fn a_name_the_module_never_grants_is_dropped_not_fatal() {
    // A human-only or native tool in the user's list must not fail every
    // registration of the subject; it simply is never auto-approved.
    let granted = grants(&posture(
        &["device_crypto", "native_secret_grant", "policy_change", "gmail_send", "  "],
        &[("flow", "sandbox_host_network"), ("flow", "http")],
    ));
    assert_eq!(granted.tools, ["gmail_send"]);
    assert_eq!(granted.flows.len(), 1);
    assert_eq!(granted.flows[0].tool, "http");
    assert!(granted.is_valid());
}

#[test]
fn grants_never_exceed_the_modules_bounds() {
    let many: Vec<String> = (0..MAX_TOOL_GRANTS2 + 10).map(|n| format!("tool_{n}")).collect();
    let granted = grants(&Posture {
        tool_grants: many,
        ..Posture::default()
    });
    assert_eq!(granted.tools.len(), MAX_TOOL_GRANTS2);
    assert!(granted.is_valid());
}

#[test]
fn a_subject_key_distinguishes_user_workspace_and_agent() {
    let a = subject_key(&subject("agent"));
    assert_ne!(a, subject_key(&subject("agent#forced")));
    assert_ne!(
        a,
        subject_key(&SubjectScope {
            workspace: "/other".into(),
            ..subject("agent")
        })
    );
}

#[test]
fn the_scope_id_is_the_subjects_and_looks_like_the_module_expects() {
    let one = entry("agent");
    let again = entry("agent");
    let forced = entry("agent#forced");
    assert_eq!(one.scope_id.len(), 32);
    assert!(one.scope_id.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_eq!(one.scope_id, again.scope_id);
    assert_ne!(one.scope_id, forced.scope_id);
}

#[tokio::test]
async fn a_bound_handle_resolves_the_subject_for_callbacks() {
    let bound = entry("handle-test-agent");
    let handle = SubjectScope {
        user: "u".repeat(64),
        workspace: "w".repeat(64),
        agent: "handle-test-agent".repeat(4),
    };
    assert!(by_handle(&handle).await.is_none(), "unknown until bound");
    bound.bind(handle.clone()).await;
    assert_eq!(bound.handle(), Some(&handle));
    let found = by_handle(&handle).await.expect("bound");
    assert!(Arc::ptr_eq(&found, &bound));
    // Binding again keeps the first handle.
    bound.bind(subject("other")).await;
    assert_eq!(bound.handle(), Some(&handle));
}

#[test]
fn a_sweep_is_due_once_per_interval() {
    let scope = entry("agent");
    assert!(scope.sweep_due(std::time::Duration::from_secs(30)));
    assert!(!scope.sweep_due(std::time::Duration::from_secs(30)));
    assert!(scope.sweep_due(std::time::Duration::ZERO));
}
