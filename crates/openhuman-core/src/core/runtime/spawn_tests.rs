use super::*;
use crate::core::runtime::{ContextOverlay, DomainSet};

fn agent_context(agent: &str) -> std::sync::Arc<CoreContext> {
    CoreContext::for_test(DomainSet::full(), None).derive_with(
        ContextOverlay::new(
            crate::config::Config::default(),
            DomainSet::full(),
            Default::default(),
        )
        .session_agent(agent),
    )
}

fn seen() -> Option<String> {
    CoreContext::current().and_then(|c| c.session_agent().map(str::to_owned))
}

#[tokio::test]
async fn a_bare_spawn_loses_the_scope_and_spawn_scoped_keeps_it() {
    let ctx = agent_context("u-asha");
    CoreContext::scope(ctx, async {
        let bare = tokio::spawn(async { seen() }).await.unwrap();
        assert_ne!(
            bare.as_deref(),
            Some("u-asha"),
            "a bare spawn falls back to the default context"
        );

        let scoped = spawn_scoped(async { seen() }).await.unwrap();
        assert_eq!(scoped.as_deref(), Some("u-asha"));
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_blocking_scoped_keeps_the_context() {
    let ctx = agent_context("u-ravi");
    let agent = CoreContext::scope(ctx, async {
        spawn_blocking_scoped(|| {
            CoreContext::current().and_then(|c| c.session_agent().map(str::to_owned))
        })
        .await
        .unwrap()
    })
    .await;
    assert_eq!(agent.as_deref(), Some("u-ravi"));
}

#[test]
fn scoping_a_large_task_keeps_its_wrapper_bounded() {
    // Background pollers capture large async state. Carrying that state
    // inline through both task-local wrappers multiplies the moves on the
    // spawning thread, including the CLI's ordinary process-main stack.
    let payload = [7u8; 256 * 1024];
    let task = async move {
        tokio::task::yield_now().await;
        std::hint::black_box(payload)[0]
    };
    let wrapped = scoped(task);
    assert!(
        std::mem::size_of_val(&wrapped) < 64 * 1024,
        "context propagation must not duplicate a large task inline ({} bytes)",
        std::mem::size_of_val(&wrapped),
    );
}
