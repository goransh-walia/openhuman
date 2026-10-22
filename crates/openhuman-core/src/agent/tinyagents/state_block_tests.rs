use super::*;

use tinyagents_harness::store::InMemoryStore;

#[test]
fn only_root_orchestrator_turns_get_the_block() {
    assert!(applies(false, Some("orchestrator")));
    assert!(!applies(true, Some("orchestrator")), "a sub-agent run");
    assert!(!applies(false, Some("planner")));
    assert!(!applies(false, None));
}

#[test]
fn install_needs_the_scope_and_a_store() {
    let store: Arc<dyn Store> = Arc::new(InMemoryStore::default());
    let installed = |store: Option<Arc<dyn Store>>, is_subagent: bool, agent: Option<&str>| {
        let mut harness = AgentHarness::<(), ()>::new();
        let before = harness.middleware().len();
        install(
            &mut harness,
            store.map(|store| (store, "standalone".to_string())),
            is_subagent,
            agent,
            false,
        );
        harness.middleware().len() - before
    };
    assert_eq!(
        installed(Some(store.clone()), false, Some("orchestrator")),
        1
    );
    assert_eq!(installed(None, false, Some("orchestrator")), 0);
    assert_eq!(
        installed(Some(store.clone()), true, Some("orchestrator")),
        0
    );
    assert_eq!(installed(Some(store), false, Some("planner")), 0);
}

/// A turn that suppresses the thread goal (`suppress_active_goal`) gets no
/// block: the block would show the goal it must not see.
#[test]
fn a_turn_that_suppresses_the_goal_gets_no_block() {
    let store: Arc<dyn Store> = Arc::new(InMemoryStore::default());
    let installed = |goal_suppressed: bool| {
        let mut harness = AgentHarness::<(), ()>::new();
        let before = harness.middleware().len();
        install(
            &mut harness,
            Some((store.clone(), "standalone".to_string())),
            false,
            Some("orchestrator"),
            goal_suppressed,
        );
        harness.middleware().len() - before
    };
    assert_eq!(installed(false), 1);
    assert_eq!(installed(true), 0);
}

#[tokio::test]
async fn a_finished_plan_hands_its_notes_over_at_the_end_of_the_turn() {
    use tinyagents_harness_tools::scratchpad::ScratchpadUpdate;
    use tinyagents_harness_tools::todos::{store as todo_store, TodoItem};

    let workspace = tempfile::tempdir().unwrap();
    let store = crate::storage::sessions::stores_or_ephemeral(workspace.path()).kv;
    todo_store::replace(&store, "t", vec![TodoItem::new("Write the migration")])
        .await
        .unwrap();
    scratchpad_store::apply(
        &store,
        "t",
        &ScratchpadUpdate {
            facts: vec!["the table is auth_users".to_string()],
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert!(close_finished_plan(workspace.path(), None).await.is_none());
    assert!(
        close_finished_plan(workspace.path(), Some("t"))
            .await
            .is_none(),
        "the plan is still open"
    );
    todo_store::complete(&store, "t", 1).await.unwrap();
    let finished = close_finished_plan(workspace.path(), Some("t"))
        .await
        .expect("the plan finished with notes");
    assert_eq!(finished.plan.notes.facts, ["the table is auth_users"]);
    assert_eq!(finished.goal, None, "the thread has no goal");
    assert!(
        close_finished_plan(workspace.path(), Some("t"))
            .await
            .is_none(),
        "handed over once"
    );
}
