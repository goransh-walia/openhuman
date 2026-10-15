//! The host's binding of the `tinyflows-schedule` runtime: the scope a
//! scheduler acts under, its store port over the cron documents, and the
//! scheduler value the poll loop, Run Now and the hosted-profile sweep drive.
//!
//! The runtime (`tinyflows_schedule::runtime::Scheduler`) owns timing: the poll
//! pass, bounded concurrency, process-wide run claims, retry and backoff,
//! rescheduling and single-flight skips. This module only supplies the ports.
//! Every port call runs under the [`CoreContext`] the scheduler was built in,
//! because the runtime spawns its tasks without the ambient scope the host's
//! stores, config and agent turns resolve through.

use super::executor::HostExecutor;
use super::run_delivery::{HostDelivery, HostTriggers};
use crate::config::Config;
use crate::core::runtime::CoreContext;
use crate::security::SecurityPolicy;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::future::Future;
use std::sync::{Arc, RwLock, Weak};
use tinyflows_schedule::runtime::{
    Completion, JobStore, RunClaim, RunRecord, RuntimePorts, Scheduler, SchedulerConfig,
    SystemClock,
};
use tinyflows_schedule::types::CronJob;

/// The claim scope every host scheduler and every manual run shares: job ids
/// are unique across agents, so Run Now, the run tool and the poll loop of any
/// agent refuse to start a job another of them is running.
pub(crate) const CLAIM_SCOPE: &str = "openhuman";

/// What every port of one scheduler shares: the config it acts on, the
/// security policy, and the context it was built in.
#[derive(Clone)]
pub(crate) struct Binding {
    config: Arc<RwLock<Config>>,
    pub(super) security: Arc<SecurityPolicy>,
    context: Option<Weak<CoreContext>>,
}

impl Binding {
    /// A binding for the context the caller is running in.
    pub(crate) fn current(config: &Config, security: Arc<SecurityPolicy>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config.clone())),
            security,
            context: CoreContext::scoped().map(|ctx| Arc::downgrade(&ctx)),
        }
    }

    /// The config as of the latest tick.
    pub(crate) fn config(&self) -> Config {
        self.config
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn set_config(&self, config: &Config) {
        *self
            .config
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = config.clone();
    }

    /// The dispatch context the bound context stands for: the bound one, or
    /// (when it has since been dropped, or none was bound) the calling task's.
    pub(crate) fn dispatch_ctx(&self) -> crate::core::runtime::DispatchCtx {
        crate::core::runtime::DispatchCtx::from_weak(self.context.as_ref())
    }

    /// Runs `fut` under the bound context. A context that has since been
    /// dropped (its agent is no longer live) runs `fut` as the caller is.
    pub(crate) async fn within<F: Future>(&self, fut: F) -> F::Output {
        match self.context.as_ref().and_then(Weak::upgrade) {
            Some(ctx) => CoreContext::scope(ctx, fut).await,
            None => fut.await,
        }
    }
}

/// `JobStore` over the cron documents of the bound context's scope
/// ([`crate::cron::store`]), refusing the jobs operator limits
/// ([`crate::cron::limits`]) no longer allow to run.
pub(super) struct HostStore {
    binding: Binding,
}

impl HostStore {
    fn keep_runnable(&self, config: &Config, jobs: Vec<CronJob>) -> Vec<CronJob> {
        jobs.into_iter()
            .filter(|job| match crate::cron::limits::check_run(config, job) {
                Ok(()) => true,
                Err(error) => {
                    tracing::warn!(job_id = %job.id, %error, "[cron:dispatch] job refused by operator limits");
                    false
                }
            })
            .collect()
    }
}

#[async_trait]
impl JobStore for HostStore {
    async fn due_jobs(&self, now: DateTime<Utc>) -> Result<Vec<CronJob>> {
        self.binding
            .within(async {
                let config = self.binding.config();
                let jobs = crate::cron::store::documents(&config)?
                    .due_jobs(now)
                    .await?;
                Ok(self.keep_runnable(&config, jobs))
            })
            .await
    }

    async fn due_jobs_excluding(
        &self,
        now: DateTime<Utc>,
        excluded: &[String],
    ) -> Result<Vec<CronJob>> {
        self.binding
            .within(async {
                let config = self.binding.config();
                let jobs = crate::cron::store::documents(&config)?
                    .due_jobs_excluding(now, excluded)
                    .await?;
                Ok(self.keep_runnable(&config, jobs))
            })
            .await
    }

    async fn claim_slot(&self, job: &CronJob, next_run: DateTime<Utc>) -> Result<()> {
        self.binding
            .within(async {
                let config = self.binding.config();
                let claimed = crate::cron::store::documents(&config)?
                    .claim_slot(job, next_run)
                    .await?;
                anyhow::ensure!(
                    claimed,
                    "schedule occurrence was removed, edited or already claimed"
                );
                tracing::trace!(job_id = %job.id, %next_run, "[cron:dispatch] slot claimed");
                Ok(())
            })
            .await
    }

    async fn record_run(&self, record: RunRecord) -> Result<()> {
        self.binding
            .within(async {
                let config = self.binding.config();
                crate::cron::store::documents(&config)?
                    .record_claimed_run(&record)
                    .await
            })
            .await
    }

    async fn complete(&self, job: &CronJob, completion: Completion) -> Result<()> {
        self.binding
            .within(async {
                let config = self.binding.config();
                crate::cron::store::documents(&config)?
                    .complete_run(job, &completion)
                    .await
            })
            .await
    }
}

/// The runtime's retry settings from the operator's reliability config.
pub(crate) fn scheduler_config(config: &Config) -> SchedulerConfig {
    SchedulerConfig {
        poll_secs: config.reliability.scheduler_poll_secs,
        max_concurrent: config.scheduler.max_concurrent,
        retries: config.reliability.scheduler_retries,
        backoff_ms: config.reliability.provider_backoff_ms,
    }
}

/// The runtime's ports for `binding`.
pub(crate) fn ports(binding: &Binding) -> RuntimePorts {
    RuntimePorts {
        store: Arc::new(HostStore {
            binding: binding.clone(),
        }),
        executor: Arc::new(HostExecutor::new(binding.clone())),
        delivery: Arc::new(HostDelivery::new(binding.clone())),
        triggers: Arc::new(HostTriggers::new(binding.clone())),
        clock: Arc::new(SystemClock),
    }
}

/// Claims `job_id` for a manual run, or `None` when a scheduled or other
/// manual run holds it. Released when the claim drops.
pub(crate) fn try_acquire_run(job_id: &str) -> Option<RunClaim> {
    RunClaim::acquire(CLAIM_SCOPE, job_id)
}

/// One scope's scheduler: polls its store, dispatches under its context and
/// owns the tasks it started (dropping it aborts them).
pub(crate) struct JobDispatcher {
    scheduler: Scheduler,
    binding: Binding,
}

impl JobDispatcher {
    /// A dispatcher for the context the caller is running in.
    pub(crate) fn new(config: &Config, security: &Arc<SecurityPolicy>) -> Self {
        let binding = Binding::current(config, Arc::clone(security));
        Self {
            scheduler: Scheduler::new(CLAIM_SCOPE, scheduler_config(config), ports(&binding)),
            binding,
        }
    }

    /// Read the due jobs and dispatch them; the count dispatched. Errors when
    /// the store cannot be read.
    pub(crate) async fn tick(&mut self, config: &Config) -> Result<usize> {
        self.binding.set_config(config);
        self.scheduler.tick().await
    }

    /// Claim and start `jobs`, returning at once.
    pub(crate) async fn dispatch(&mut self, jobs: Vec<CronJob>) -> Result<usize> {
        self.scheduler.dispatch(jobs).await
    }

    /// Wait for every dispatched job to finish.
    pub(crate) async fn drain(&mut self) {
        if let Err(error) = self.scheduler.drain().await {
            tracing::warn!(%error, "[cron:dispatch] a dispatched job failed");
        }
    }
}
