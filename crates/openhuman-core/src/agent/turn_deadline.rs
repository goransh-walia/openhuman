//! Wall-clock deadline of one top-level agent turn.
//!
//! A web-chat turn runs under an outer backstop (`web_chat::ops::turn_guards`)
//! that drops the turn future when it elapses. Dropping it discards
//! everything the turn did. Production showed ~55 orchestrator turns a week
//! dying this way at exactly 900s, after 50–98 model calls, with nothing
//! delivered. They died there because the harness's own turn ceiling (3600s,
//! `tinyagents::turn_policy`) was later than the backstop, so the harness
//! never got the chance to stop the turn itself.
//!
//! The deadline value (`tinyagents_harness::turn_deadline::TurnDeadline`)
//! owns the two points the harness derives from the backstop; this module
//! keeps the host side (env override, task-local scope):
//!
//! * **wind-down** (`backstop - min(120s, backstop/5)`, 780s by default): the
//!   harness pauses the run at the next safe point, after a model call or a
//!   tool result. The driver then writes the grounded close from the work
//!   already in hand. The margin before the backstop pays for that close.
//! * **hard stop** (`backstop - min(60s, backstop/10)`, 840s by default): the
//!   harness `max_wall_clock_ms` is clamped to this. A single tool call that
//!   runs across the wind-down point is then interrupted by the harness's own
//!   typed `Timeout` before the backstop drops the turn.
//!
//! For every positive backstop, `wind_down < hard_stop < backstop`
//! (see `turn_deadline_tests.rs`).
//!
//! The web-turn guard scopes the deadline on a task-local
//! ([`with_turn_deadline`]). The top-level session turn
//! (`OpenHumanSessionHost::turn_with_origin`) copies it onto its explicit run
//! context. Turns with no backstop, such as other channels or the CLI, carry
//! `None` and keep their previous behaviour.

use std::future::Future;
use std::time::Duration;

use tinyagents_harness::turn_deadline::{parse_backstop, TurnDeadline};

/// Environment override for the backstop, in seconds; `0` disables it.
pub const TURN_BACKSTOP_ENV: &str = "OPENHUMAN_WEB_TURN_TIMEOUT_SECS";

/// Time kept back from the post-loop grounded close so the turn can commit and
/// emit its reply before the backstop drops it.
const CLOSE_RESERVE: Duration = Duration::from_secs(10);

/// How long the post-loop grounded close may run as of `now`: the backstop
/// remainder minus [`CLOSE_RESERVE`], zero once that has passed (#7716).
/// Host-side until the harness `TurnDeadline` grows the same accessor.
pub fn close_budget(deadline: &TurnDeadline, now: std::time::Instant) -> Duration {
    deadline
        .backstop()
        .saturating_sub(deadline.elapsed_at(now))
        .saturating_sub(CLOSE_RESERVE)
}

/// The backstop in force for this process (env [`TURN_BACKSTOP_ENV`]).
pub fn configured_backstop() -> Option<Duration> {
    parse_backstop(std::env::var(TURN_BACKSTOP_ENV).ok().as_deref())
}

tokio::task_local! {
    static CURRENT_TURN_DEADLINE: TurnDeadline;
}

/// The deadline scoped on the current task, if any.
pub fn current() -> Option<TurnDeadline> {
    CURRENT_TURN_DEADLINE.try_with(|deadline| *deadline).ok()
}

/// Run `future` with `deadline` scoped on the task (a no-op scope for `None`).
pub async fn with_turn_deadline<F: Future>(deadline: Option<TurnDeadline>, future: F) -> F::Output {
    match deadline {
        Some(deadline) => CURRENT_TURN_DEADLINE.scope(deadline, future).await,
        None => future.await,
    }
}

#[cfg(test)]
#[path = "turn_deadline_tests.rs"]
mod tests;
