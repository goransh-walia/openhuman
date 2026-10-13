//! The high-frequency agent-job warning. Persisting a finished run (history,
//! one-shot termination, rescheduling) is the schedule runtime's
//! (`tinyflows_schedule::runtime`).

use chrono::Utc;
use {
    tinyflows_schedule::schedule::runs_closer_than, tinyflows_schedule::schedule::TooFrequent,
    tinyflows_schedule::schedule::MIN_AGENT_JOB_INTERVAL, tinyflows_schedule::types::CronJob,
    tinyflows_schedule::types::JobType,
};

/// Why an agent job counts as scheduled more often than every five minutes,
/// if it does. `None` for shell and flow jobs, for one-shot `At` schedules,
/// for anything at or above [`MIN_AGENT_JOB_INTERVAL`], and for an expression
/// that cannot be read (that is not evidence of anything).
///
/// Returns the verdict rather than only logging it, because the verdict is
/// the part worth testing: the `Schedule::Cron` arm used to measure the gap
/// between the next run after now and the next run after now plus one
/// second — the same instant unless a run fell inside that second — so every
/// cron-scheduled agent job warned, and no test could see it because a
/// function that only warns has nothing to assert. The gap is now the
/// shortest one between consecutive runs (`runs_closer_than`), so an
/// irregular expression is judged by its tightest pair and not by whichever
/// pair happens to follow the instant of the check.
pub(super) fn agent_job_too_frequent(job: &CronJob) -> Option<TooFrequent> {
    if !matches!(job.job_type, JobType::Agent) {
        return None;
    }
    runs_closer_than(&job.schedule, Utc::now(), MIN_AGENT_JOB_INTERVAL)
}

/// New agent jobs cannot be created below the floor (`validate_agent_schedule`
/// rejects them), so a job that trips this predates the rule. It keeps
/// running — silently skipping runs would turn its schedule into a lie — and
/// this line is the operator's signal to edit it.
pub(super) fn warn_if_high_frequency_agent_job(job: &CronJob) {
    if let Some(too_frequent) = agent_job_too_frequent(job) {
        tracing::warn!(
            "Cron agent job '{}' is scheduled more frequently than every 5 minutes: it \
             {too_frequent}. It was created before that floor was enforced; edit its \
             schedule to silence this.",
            job.id
        );
    }
}
