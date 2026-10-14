//! Explicit host environment for one agent turn.
//!
//! An entry point that drives a turn on behalf of a host (the embed facade, the
//! web channel, a scheduler) owns a handful of per-turn values the harness has
//! to honour: the shared model-spend budget, a narrower delegation depth, the
//! caller's cancellation, the wall-clock backstop, hook callbacks, and whether
//! the host's tool belt is authoritative over a restored transcript. They used
//! to ride ambient task-locals that every layer below re-read, which silently
//! stopped working across a `tokio::spawn` and could not be seen from a
//! signature. This value is handed to the session instead
//! ([`OpenHumanSessionHost::set_host_env`](crate::agent::OpenHumanSessionHost::set_host_env));
//! the session copies it onto the turn's explicit
//! [`OpenHumanRunContext`](crate::agent::tinyagents::host::OpenHumanRunContext),
//! whose `child()` hands it down the sub-agent tree.

use tinyagents_harness::cancel::CancellationToken;
use tinyagents_harness::turn_deadline::TurnDeadline;

use crate::agent::hooks::HookScope;
use crate::agent::tinyagents::budget::ModelBudget;

/// Per-turn values an entry point hands to the session it runs.
///
/// Every field defaults to "nothing narrowed", so `HostTurnEnv::default()` is a
/// turn exactly as it ran before any host policy existed.
#[derive(Clone, Default)]
pub struct HostTurnEnv {
    /// Shared model-spend ledger and per-call bounds every model call of the
    /// turn tree is charged against.
    pub model_budget: Option<ModelBudget>,
    /// A narrower model-directed delegation depth than the product ceiling.
    pub spawn_depth_limit: Option<usize>,
    /// The caller's cancellation, shared with every child run. `None` runs the
    /// turn under a fresh token only the session itself can fire.
    pub cancellation: Option<CancellationToken>,
    /// The wall-clock backstop the harness winds down before.
    pub turn_deadline: Option<TurnDeadline>,
    /// A cap on real tool invocations, never wider than the product default.
    pub tool_call_limit: Option<usize>,
    /// Agent/turn hook callbacks added after the runtime's global ones.
    pub hooks: HookScope,
    /// The host's tool belt for this turn is authoritative: tool declarations
    /// recorded in a restored transcript must not bring a revoked tool back.
    pub fresh_tool_snapshot: bool,
}

impl std::fmt::Debug for HostTurnEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostTurnEnv")
            .field("model_budget", &self.model_budget.is_some())
            .field("spawn_depth_limit", &self.spawn_depth_limit)
            .field("cancellation", &self.cancellation.is_some())
            .field("turn_deadline", &self.turn_deadline.is_some())
            .field("tool_call_limit", &self.tool_call_limit)
            .field("hooks_empty", &self.hooks.is_empty())
            .field("fresh_tool_snapshot", &self.fresh_tool_snapshot)
            .finish()
    }
}

#[cfg(test)]
#[path = "turn_env_tests.rs"]
mod tests;
