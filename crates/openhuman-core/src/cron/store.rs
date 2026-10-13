//! Cron persistence: a thin host binding of `tinyflows_drivers::schedule::CronDocuments`.
//!
//! Jobs, run history and per-job policy live on the storage ports, in the
//! acting agent's scope of the storage backend ([`crate::storage`]). tinyflows no
//! longer has a SQLite store of its own: the `jobs.db` it wrote is imported
//! once, read-only, when the backend is installed ([`crate::storage::import`]).
//!
//! This module only turns the host [`Config`] into the store's limits
//! (run-history cap, due-job batch size), so callers keep passing `&Config`.

use crate::config::Config;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tinyflows_drivers::schedule::CronDocuments;
use tinyflows_schedule::DeliveryStatus;
use tinyflows_schedule::{
    CronJob, CronJobPatch, CronRun, DeliveryConfig, JobType, Schedule, SessionTarget,
};

use super::limits;
use tinyflows_schedule::AgentJobSpec;

/// The cron database tinyflows used to write, imported on first use.
pub fn legacy_db_path(config: &Config) -> std::path::PathBuf {
    crate::storage::import::tinyflows::legacy_jobs_db(&crate::core::runtime::agent_scope_dir(
        config,
    ))
}

/// The cron job store for this call, in the acting agent's scope.
pub(crate) fn documents(config: &Config) -> Result<CronDocuments> {
    let opened = crate::storage::local::open("cron::store", &config.workspace_dir)?;
    Ok(
        CronDocuments::new(std::sync::Arc::clone(opened.scoped()?.documents()))
            .with_limits(config.cron.max_run_history, config.scheduler.max_tasks),
    )
}

/// Runs a document-store call from this synchronous API.
fn run<T: Send + 'static>(
    future: impl std::future::Future<Output = Result<T>> + Send + 'static,
) -> Result<T> {
    crate::storage::block_on_anyhow(future)
}

pub fn add_job(config: &Config, expression: &str, command: &str) -> Result<CronJob> {
    let schedule = Schedule::Cron {
        expr: expression.to_string(),
        tz: None,
        active_hours: None,
    };
    limits::check_new(config, &JobType::Shell, &schedule, None)?;
    let docs = documents(config)?;
    let (expression, command) = (expression.to_string(), command.to_string());
    run(async move { docs.add_job(&expression, &command).await })
}

pub fn add_shell_job(
    config: &Config,
    name: Option<String>,
    schedule: Schedule,
    command: &str,
) -> Result<CronJob> {
    limits::check_new(config, &JobType::Shell, &schedule, None)?;
    let docs = documents(config)?;
    let command = command.to_string();
    run(async move { docs.add_shell_job(name, schedule, &command).await })
}

#[allow(clippy::too_many_arguments)]
pub fn add_agent_job(
    config: &Config,
    name: Option<String>,
    schedule: Schedule,
    prompt: &str,
    session_target: SessionTarget,
    model: Option<String>,
    delivery: Option<DeliveryConfig>,
    delete_after_run: bool,
) -> Result<CronJob> {
    limits::check_new(config, &JobType::Agent, &schedule, delivery.as_ref())?;
    let docs = documents(config)?;
    let prompt = prompt.to_string();
    run(async move {
        docs.add_agent_job(
            name,
            schedule,
            &prompt,
            session_target,
            model,
            delivery,
            delete_after_run,
        )
        .await
    })
}

/// Like [`add_agent_job`] but accepts an optional built-in agent definition
/// ID and the initial enabled state.
#[allow(clippy::too_many_arguments)]
pub fn add_agent_job_with_definition(
    config: &Config,
    name: Option<String>,
    schedule: Schedule,
    prompt: &str,
    session_target: SessionTarget,
    model: Option<String>,
    delivery: Option<DeliveryConfig>,
    delete_after_run: bool,
    agent_id: Option<String>,
    enabled: bool,
) -> Result<CronJob> {
    limits::check_new(config, &JobType::Agent, &schedule, delivery.as_ref())?;
    let docs = documents(config)?;
    let prompt = prompt.to_string();
    run(async move {
        docs.add_agent_job_with_definition(
            name,
            schedule,
            &prompt,
            session_target,
            model,
            delivery,
            delete_after_run,
            agent_id,
            enabled,
        )
        .await
    })
}

/// Adds an agent job described by `spec`, including its origin conversation.
pub fn add_agent_job_from_spec(config: &Config, spec: AgentJobSpec) -> Result<CronJob> {
    limits::check_new(
        config,
        &JobType::Agent,
        &spec.schedule,
        spec.delivery.as_ref(),
    )?;
    let docs = documents(config)?;
    run(async move { docs.add_agent_job_from_spec(spec).await })
}

/// Registers (idempotently) the cron job that fires a flow's `schedule`
/// trigger; see `CronDocuments::add_flow_schedule_job`.
pub fn add_flow_schedule_job(
    config: &Config,
    flow_id: &str,
    schedule: Schedule,
) -> Result<CronJob> {
    limits::check_new(config, &JobType::Flow, &schedule, None)?;
    let docs = documents(config)?;
    let flow_id = flow_id.to_string();
    run(async move { docs.add_flow_schedule_job(&flow_id, schedule).await })
}

pub fn find_flow_schedule_job(config: &Config, flow_id: &str) -> Result<Option<CronJob>> {
    let docs = documents(config)?;
    let flow_id = flow_id.to_string();
    run(async move { docs.find_flow_schedule_job(&flow_id).await })
}

pub fn list_jobs(config: &Config) -> Result<Vec<CronJob>> {
    let docs = documents(config)?;
    run(async move { docs.list_jobs().await })
}

pub fn get_job(config: &Config, job_id: &str) -> Result<CronJob> {
    let docs = documents(config)?;
    let job_id = job_id.to_string();
    run(async move { docs.get_job(&job_id).await })
}

pub fn remove_job(config: &Config, id: &str) -> Result<()> {
    let docs = documents(config)?;
    let owned = id.to_string();
    // The job's policy is a field of the job, so it goes with it.
    run(async move { docs.remove_job(&owned).await })?;
    println!("✅ Removed cron job {id}");
    Ok(())
}

/// Deletes every cron job in the workspace (E2E `openhuman.test_reset`).
pub fn clear_all_jobs(config: &Config) -> Result<usize> {
    let docs = documents(config)?;
    run(async move { docs.clear_all_jobs().await })
}

/// Removes duplicate jobs sharing a `name`, keeping the one with most history.
pub fn dedup_named_jobs(config: &Config) -> Result<usize> {
    let docs = documents(config)?;
    run(async move { docs.dedup_named_jobs().await })
}

pub fn due_jobs(config: &Config, now: DateTime<Utc>) -> Result<Vec<CronJob>> {
    let docs = documents(config)?;
    run(async move { docs.due_jobs(now).await })
}

pub fn update_job(config: &Config, job_id: &str, patch: CronJobPatch) -> Result<CronJob> {
    limits::check_patch(config, &patch)?;
    let docs = documents(config)?;
    let job_id = job_id.to_string();
    run(async move { docs.update_job(&job_id, patch).await })
}

pub fn record_last_run(
    config: &Config,
    job_id: &str,
    finished_at: DateTime<Utc>,
    success: bool,
    output: &str,
) -> Result<()> {
    let docs = documents(config)?;
    let (job_id, output) = (job_id.to_string(), output.to_string());
    run(async move {
        docs.record_last_run(&job_id, finished_at, success, &output)
            .await
    })
}

pub fn reschedule_after_run(
    config: &Config,
    job: &CronJob,
    success: bool,
    output: &str,
) -> Result<()> {
    let docs = documents(config)?;
    let (job, output) = (job.clone(), output.to_string());
    run(async move { docs.reschedule_after_run(&job, success, &output).await })
}

pub fn record_run(
    config: &Config,
    job_id: &str,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    status: &str,
    output: Option<&str>,
    duration_ms: i64,
) -> Result<()> {
    let docs = documents(config)?;
    let (job_id, status, output) = (
        job_id.to_string(),
        status.to_string(),
        output.map(str::to_string),
    );
    run(async move {
        docs.record_run(
            &job_id,
            started_at,
            finished_at,
            &status,
            output.as_deref(),
            duration_ms,
        )
        .await
    })
}

/// [`record_run`] plus the outcome of delivering the run's result.
#[allow(clippy::too_many_arguments)]
pub fn record_run_with_delivery(
    config: &Config,
    job_id: &str,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    status: &str,
    output: Option<&str>,
    duration_ms: i64,
    delivery_status: Option<DeliveryStatus>,
) -> Result<()> {
    let docs = documents(config)?;
    let (job_id, status, output) = (
        job_id.to_string(),
        status.to_string(),
        output.map(str::to_string),
    );
    run(async move {
        docs.record_run_with_delivery(
            &job_id,
            started_at,
            finished_at,
            &status,
            output.as_deref(),
            duration_ms,
            delivery_status,
        )
        .await
    })
}

/// Removes "queued" placeholder rows so only the real result row remains.
pub fn delete_queued_runs(config: &Config, job_id: &str) -> Result<usize> {
    let docs = documents(config)?;
    let job_id = job_id.to_string();
    run(async move { docs.delete_queued_runs(&job_id).await })
}

pub fn list_runs(config: &Config, job_id: &str, limit: usize) -> Result<Vec<CronRun>> {
    let docs = documents(config)?;
    let job_id = job_id.to_string();
    run(async move { docs.list_runs(&job_id, limit).await })
}

/// Moves a job's next run to `at`, which no production path can do directly
/// (a patch reschedules from the schedule, not to a given instant). Writes the
/// three forms the store keeps of the instant, so the due-job query sees it.
#[cfg(test)]
pub(crate) fn force_next_run_for_test(
    config: &Config,
    job_id: &str,
    at: DateTime<Utc>,
) -> Result<()> {
    use tinystoragedrivers::Precondition;
    let opened = crate::storage::local::open("cron::store", &config.workspace_dir)?;
    let docs = std::sync::Arc::clone(opened.scoped()?.documents());
    let job_id = job_id.to_string();
    run(async move {
        let stored = docs
            .get("cron_jobs", &job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("no job {job_id}"))?;
        let mut doc = stored.doc;
        doc["next_run"] = serde_json::json!(at.to_rfc3339());
        doc["next_run_ms"] = serde_json::json!(at.timestamp_millis());
        doc["next_run_ns"] = serde_json::json!(at.timestamp_nanos_opt().unwrap_or(i64::MAX));
        docs.put("cron_jobs", &job_id, doc, Precondition::None)
            .await?;
        Ok(())
    })
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
