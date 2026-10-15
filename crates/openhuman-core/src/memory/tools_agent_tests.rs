use super::*;
use crate::core::runtime::{context::CoreContext, ContextOverlay, DomainSet};
use crate::tools::toolpacks::ToolGroups;

fn agent(parent: &Arc<CoreContext>, id: &str) -> Arc<CoreContext> {
    parent.derive_with(
        ContextOverlay::new(
            crate::config::Config::default(),
            DomainSet::kernel(),
            ToolGroups::none(),
        )
        .session_agent(id),
    )
}

fn citation(id: &str) -> TurnCitation {
    TurnCitation {
        id: id.to_string(),
        key: "learning".to_string(),
        namespace: None,
        score: None,
        timestamp: String::new(),
        snippet: String::new(),
    }
}

#[tokio::test]
async fn turn_citations_stay_with_the_agent_that_recorded_them() {
    let root = CoreContext::for_test(DomainSet::full(), None);
    let alpha = agent(&root, "alpha");
    let beta = agent(&root, "beta");
    let thread = "shared-thread";

    CoreContext::scope(Arc::clone(&alpha), async {
        record_pack_citations(thread, vec![citation("alpha-cite")]);
    })
    .await;
    CoreContext::scope(Arc::clone(&beta), async {
        record_pack_citations(thread, vec![citation("beta-cite")]);
    })
    .await;

    let from_beta = CoreContext::scope(beta, async { take_turn_citations(thread) }).await;
    let from_alpha = CoreContext::scope(alpha, async { take_turn_citations(thread) }).await;

    assert_eq!(
        from_alpha.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["alpha-cite"]
    );
    assert_eq!(
        from_beta.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["beta-cite"]
    );
}

/// A tool context for a run that acts on memory as `identity`.
fn tool_context_acting_as(
    identity: crate::memory::scope::MemoryIdentity,
) -> tinyagents_harness::tool::ToolExecutionContext {
    let mut host = crate::agent::tinyagents::host::OpenHumanRunContext::new();
    host.memory_identity = Some(identity);
    let run = host.into_tinyagents(tinyagents_harness::context::RunConfig::new("identity-test"));
    tinyagents_harness::tool::ToolExecutionContext::from_run_context(
        &run,
        tinyagents_harness::ids::CallId::new("identity-test-call"),
    )
}

/// The memory tool acts as the identity its run carries, not as whatever task
/// it happens to be polled on: a team member's learnings land in the team's
/// layout under the member's own memory agent id.
#[test]
fn gather_acts_as_the_identity_the_run_carries() {
    let config = Config::default();
    let context = tool_context_acting_as(crate::memory::scope::MemoryIdentity::team_member(
        "acme", "scout",
    ));
    let facts = CallFacts::gather(&config, Some(&context));
    assert_eq!(facts.agent_id.as_deref(), Some("scout"));
    assert_eq!(facts.reach, Reach::subtree("team:acme".parse().unwrap()));
    assert_eq!(facts.namespace.to_string(), "team:acme");

    // A run that carries none, and direct dispatch, act as the root.
    let bare = CallFacts::gather(&config, None);
    assert_eq!(
        bare.agent_id.as_deref(),
        Some(super::super::scope::DEFAULT_AGENT_ID)
    );
    assert!(bare.reach.at.is_root());
}
