//! Host side of the wall-clock deadline of one top-level agent turn: the env
//! override. The deadline itself (`close_budget`
//! included) is `tinyagents_harness::turn_deadline::TurnDeadline`.
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
//! keeps the host side (the env override):
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
//! (see `turn_backstop_tests.rs`).
//!
//! The web-turn guard builds the deadline and hands it to the session it runs
//! (`HostTurnEnv::turn_deadline`). The top-level session turn
//! (`OpenHumanSessionHost::turn_with_origin`) copies it onto its explicit run
//! context. Turns with no backstop, such as other channels or the CLI, carry
//! `None` and keep their previous behaviour.

use std::time::Duration;

use tinyagents_harness::turn_deadline::{parse_backstop, TurnDeadline};

/// Environment override for the backstop, in seconds; `0` disables it.
pub const TURN_BACKSTOP_ENV: &str = "OPENHUMAN_WEB_TURN_TIMEOUT_SECS";

/// The backstop in force for this process (env [`TURN_BACKSTOP_ENV`]).
pub fn configured_backstop() -> Option<Duration> {
    parse_backstop(std::env::var(TURN_BACKSTOP_ENV).ok().as_deref())
}

#[cfg(test)]
#[path = "turn_backstop_tests.rs"]
mod tests;
