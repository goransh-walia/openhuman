//! The host's `JobExecutor`: one attempt of a shell or agent job under the
//! security policy and operator limits, classified for the runtime.
//!
//! The runtime (`tinyflows_schedule::runtime`) owns the retry loop and its
//! backoff. This executor decides, per attempt, whether a failure is worth
//! another one: a policy block or a permanent user-config / billing state
//! (expired session, no credits, exhausted budget, unset key, unreachable
//! local provider) is returned `retryable: false`, and the final failure is
//! reported once through [`JobExecutor::failed`].

use super::agent_run::run_agent_job_for_run;
use super::failure_classification::{
    is_api_key_unset_failure, is_budget_exhausted_failure, is_insufficient_credits_failure,
    is_local_provider_unreachable_failure, is_session_expired_failure, permanent_halt_message,
};
use super::host::Binding;
use super::run_record::warn_if_high_frequency_agent_job;
use super::shell_job::run_job_command;
use crate::core::bus::BUS;
use crate::core::events::DomainEvent;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use tinyflows_schedule::runtime::{Attempt, JobExecutor};
use tinyflows_schedule::types::{CronJob, JobType, SessionTarget};

/// Run states kept before the map is cleared outright (a run whose task was
/// aborted never reaches `failed`).
const CAPACITY: usize = 1024;

/// Why a run stopped retrying, which decides what its final report says.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Halt {
    #[default]
    None,
    /// Refused before running (operator limits, security policy).
    Refused,
    SessionExpired,
    LocalUnreachable,
    InsufficientCredits,
    BudgetExhausted,
    ApiKeyUnset,
}

/// What the executor remembers about a run across its attempts.
#[derive(Default)]
struct RunState {
    last_agent_error: Option<String>,
    halt: Halt,
}

pub(super) struct HostExecutor {
    binding: Binding,
    runs: Mutex<HashMap<String, RunState>>,
}

impl HostExecutor {
    pub(super) fn new(binding: Binding) -> Self {
        Self {
            binding,
            runs: Mutex::new(HashMap::new()),
        }
    }

    /// Registers `run_id`; `true` on its first attempt.
    fn begin(&self, run_id: &str) -> bool {
        let mut runs = self.runs.lock().unwrap_or_else(|e| e.into_inner());
        if runs.contains_key(run_id) {
            return false;
        }
        if runs.len() >= CAPACITY {
            runs.clear();
        }
        runs.insert(run_id.to_string(), RunState::default());
        true
    }

    fn forget(&self, run_id: &str) -> Option<RunState> {
        self.runs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(run_id)
    }

    /// Records the attempt's agent error and returns the latest one seen.
    fn note_error(&self, run_id: &str, agent_error: Option<String>) -> Option<String> {
        let mut runs = self.runs.lock().unwrap_or_else(|e| e.into_inner());
        let state = runs.entry(run_id.to_string()).or_default();
        if agent_error.is_some() {
            state.last_agent_error = agent_error;
        }
        state.last_agent_error.clone()
    }

    fn halt(&self, run_id: &str, halt: Halt) {
        let mut runs = self.runs.lock().unwrap_or_else(|e| e.into_inner());
        runs.entry(run_id.to_string()).or_default().halt = halt;
    }

    async fn attempt(&self, job: &CronJob, run_id: &str) -> Attempt {
        let config = self.binding.config();
        if self.begin(run_id) {
            warn_if_high_frequency_agent_job(job);
            BUS.publish(DomainEvent::CronJobTriggered {
                job_id: job.id.clone(),
                job_name: job.name.clone().unwrap_or_default(),
                job_type: format!("{:?}", job.job_type),
            });
        }
        // Operator limits (`cron::limits`) also bind rows stored before they
        // were switched on.
        if let Err(error) = crate::cron::limits::check_run(&config, job) {
            self.halt(run_id, Halt::Refused);
            return Attempt {
                success: false,
                output: error.to_string(),
                retryable: false,
            };
        }
        let (success, output, agent_error) = match job.job_type {
            JobType::Shell => {
                let (success, output) =
                    run_job_command(&config, self.binding.security.as_ref(), job).await;
                (success, output, None)
            }
            JobType::Agent => run_agent_job_for_run(&config, job, run_id).await,
            JobType::Flow => (
                false,
                "flow jobs run through the trigger sink, not the executor".to_string(),
                None,
            ),
        };
        if success {
            self.forget(run_id);
            return Attempt {
                success: true,
                output,
                retryable: true,
            };
        }
        let last_agent_error = self.note_error(run_id, agent_error);
        self.classify_failure(job, run_id, output, last_agent_error.as_deref())
    }

    /// Whether a failed attempt may be retried, and the output to record.
    fn classify_failure(
        &self,
        job: &CronJob,
        run_id: &str,
        output: String,
        last_agent_error: Option<&str>,
    ) -> Attempt {
        let permanent = |halt: Halt, output: String| {
            self.halt(run_id, halt);
            Attempt {
                success: false,
                output,
                retryable: false,
            }
        };
        if output.starts_with("blocked by security policy:") {
            // Deterministic policy violations are not retryable.
            return permanent(Halt::Refused, output);
        }
        let job_type = &job.job_type;
        if is_session_expired_failure(job_type, last_agent_error, &output) {
            // The inference layer already published `SessionExpired`; retries
            // cannot recover until the user re-auths (TAURI-RUST-N).
            return permanent(Halt::SessionExpired, output);
        }
        let config_halt = if is_insufficient_credits_failure(job_type, last_agent_error, &output) {
            Some((Halt::InsufficientCredits, "insufficient_credits"))
        } else if is_budget_exhausted_failure(job_type, last_agent_error, &output) {
            Some((Halt::BudgetExhausted, "budget_exceeded"))
        } else if is_api_key_unset_failure(job_type, last_agent_error, &output) {
            Some((Halt::ApiKeyUnset, "api_key_missing"))
        } else {
            None
        };
        if let Some((halt, user_error_kind)) = config_halt {
            tracing::debug!(
                job_id = %job.id,
                ?halt,
                "[cron:executor] permanent user-config halt; not retrying"
            );
            // The specific, actionable (static, leak-safe) copy replaces the
            // generic agent-failure text; the raw error chain is never shown.
            let message = permanent_halt_message(
                halt == Halt::InsufficientCredits,
                halt == Halt::BudgetExhausted,
            );
            // Also reach the UserErrorCenter so the user can fix it with no
            // chat thread open (metadata-only; #4165 / TAURI-RUST-HCK).
            publish_cron_user_error(user_error_kind);
            return permanent(halt, message.to_string());
        }
        if is_local_provider_unreachable_failure(job_type, last_agent_error, &output) {
            // A local LLM server refusing the loopback connection cannot
            // recover across the backoff loop (TAURI-RUST-12K).
            return permanent(Halt::LocalUnreachable, output);
        }
        Attempt {
            success: false,
            output,
            retryable: true,
        }
    }
}

#[async_trait]
impl JobExecutor for HostExecutor {
    async fn execute(&self, job: &CronJob, run_id: &str) -> Result<Attempt> {
        Ok(self.binding.within(self.attempt(job, run_id)).await)
    }

    /// The retries-exhausted (or permanent) report, once per failed run.
    async fn failed(&self, job: &CronJob, run_id: &str, attempt: &Attempt) {
        let state = self.forget(run_id).unwrap_or_default();
        if !matches!(job.job_type, JobType::Agent) {
            return;
        }
        match state.halt {
            Halt::None => {
                let message = state
                    .last_agent_error
                    .as_deref()
                    .unwrap_or(attempt.output.as_str());
                crate::core::observability::report_error(
                    message,
                    "cron",
                    "agent_job",
                    &[
                        ("job_id", job.id.as_str()),
                        ("agent_id", job.agent_id.as_deref().unwrap_or("none")),
                        (
                            "session_target",
                            agent_session_target_tag(&job.session_target),
                        ),
                        ("failure", "retries_exhausted"),
                    ],
                );
            }
            halt => tracing::debug!(
                job_id = %job.id,
                ?halt,
                "[cron:executor] suppressed the retries-exhausted report for a permanent halt"
            ),
        }
    }
}

pub(super) fn agent_session_target_tag(target: &SessionTarget) -> &'static str {
    match target {
        SessionTarget::Main => "main",
        SessionTarget::Isolated => "isolated",
        SessionTarget::Current => "current",
    }
}

/// Surface a permanent cron user-config / billing halt to every connected
/// client's UserErrorCenter.
///
/// Broadcasts a metadata-only `user_error` web-channel event to the `"system"`
/// room (which every socket auto-joins). The payload carries only the stable
/// `kind` token in `error_type` — one of `api_key_missing` / `insufficient_credits`
/// / `budget_exceeded`, mirroring the frontend `UserErrorKind` discriminator —
/// plus `error_source = "cron"`. It NEVER carries the raw provider body (see the
/// metadata-only rule in CLAUDE.md), so no secrets / PII leave the core.
pub(super) fn publish_cron_user_error(kind: &str) {
    tracing::debug!("[cron] action=surface_user_error kind={kind}");
    crate::web_chat::publish_web_channel_event(crate::web_chat::WebChannelEvent {
        event: "user_error".to_string(),
        client_id: "system".to_string(),
        error_type: Some(kind.to_string()),
        error_source: Some("cron".to_string()),
        ..Default::default()
    });
}
