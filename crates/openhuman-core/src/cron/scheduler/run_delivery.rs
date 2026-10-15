//! The host's `RunDelivery` and `TriggerSink` ports for the schedule runtime.
//!
//! Delivery sends a finished run's output to its chat, channel or origin and
//! raises the alert (`delivery.rs`), then announces the completion on the bus.
//! The trigger sink fires a `flow` row's schedule trigger directly in the
//! flows domain, or runs a host-owned system job, with no domain event in
//! between.

use super::delivery::deliver_run;
use super::host::Binding;
use crate::core::bus::BUS;
use crate::core::events::DomainEvent;
use anyhow::Result;
use async_trait::async_trait;
use tinyflows_schedule::runtime::{Attempt, RunDelivery, TriggerSink};
use tinyflows_schedule::types::{CronJob, DeliveryStatus};

pub(super) struct HostDelivery {
    binding: Binding,
}

impl HostDelivery {
    pub(super) fn new(binding: Binding) -> Self {
        Self { binding }
    }
}

#[async_trait]
impl RunDelivery for HostDelivery {
    async fn deliver(
        &self,
        job: &CronJob,
        run_id: &str,
        attempt: &Attempt,
    ) -> Result<DeliveryStatus> {
        self.binding
            .within(async {
                let config = self.binding.config();
                let ctx = self.binding.dispatch_ctx();
                let result =
                    deliver_run(&ctx, &config, job, run_id, &attempt.output, attempt.success).await;
                // A failed required delivery fails the run, as the runtime
                // records it.
                let delivered = match &result {
                    Ok(status) => *status != DeliveryStatus::Failed || job.delivery.best_effort,
                    Err(_) => job.delivery.best_effort,
                };
                let success = attempt.success && delivered;
                // The job ran as its agent (when it had one); a one-shot job
                // is deleted once the run completes, so subscribers learn the
                // owner here, not from the store.
                crate::cron::completion_owner::note(&ctx, &job.id);
                BUS.publish(DomainEvent::CronJobCompleted {
                    job_id: job.id.clone(),
                    success,
                    output: crate::util::truncate_with_ellipsis(&attempt.output, 512),
                });
                BUS.publish(DomainEvent::HealthChanged {
                    component: "scheduler".to_string(),
                    healthy: success,
                    message: (!success).then(|| {
                        if attempt.output.is_empty() {
                            format!("job {} failed", job.id)
                        } else {
                            crate::util::truncate_with_ellipsis(&attempt.output, 256)
                        }
                    }),
                });
                result
            })
            .await
    }
}

pub(super) struct HostTriggers {
    binding: Binding,
    /// Holds the per-flow in-flight dedupe across ticks.
    #[cfg(feature = "flows")]
    flows: std::sync::OnceLock<crate::flows::bus::FlowTriggerSubscriber>,
}

impl HostTriggers {
    pub(super) fn new(binding: Binding) -> Self {
        Self {
            binding,
            #[cfg(feature = "flows")]
            flows: std::sync::OnceLock::new(),
        }
    }
}

#[async_trait]
impl TriggerSink for HostTriggers {
    async fn schedule(&self, flow_id: &str, run_id: &str) -> Result<Attempt> {
        self.binding
            .within(async {
                if let Some(name) = crate::cron::system_jobs::system_command_name(flow_id) {
                    let config = self.binding.config();
                    let job_id = match crate::cron::store::documents(&config) {
                        Ok(docs) => docs.find_flow_schedule_job(flow_id).await.ok().flatten(),
                        Err(_) => None,
                    }
                    .map_or_else(|| flow_id.to_string(), |job| job.id);
                    return Ok(run_system_job(&job_id, name, run_id).await);
                }
                // `[components] flows = false`: schedule-triggered flows stop
                // while agent, shell and system jobs keep running. The tick is
                // answered, not retried, so the row's recurrence moves on.
                if !self.binding.dispatch_ctx().components().flows {
                    tracing::debug!(
                        target: "flows",
                        %flow_id,
                        %run_id,
                        "[cron] flow schedule trigger skipped: component 'flows' disabled"
                    );
                    return Ok(Attempt {
                        success: true,
                        output: format!("flow schedule tick skipped for flow {flow_id}: COMPONENT_DISABLED:flows"),
                        retryable: false,
                    });
                }
                tracing::info!(
                    target: "flows",
                    %flow_id,
                    %run_id,
                    "[cron] flow schedule trigger fired"
                );
                #[cfg(feature = "flows")]
                self.flows
                    .get_or_init(|| {
                        crate::flows::bus::FlowTriggerSubscriber::new(std::sync::Arc::new(
                            self.binding.config(),
                        ))
                    })
                    .dispatch_schedule_tick(flow_id)
                    .await;
                #[cfg(not(feature = "flows"))]
                tracing::warn!(%flow_id, "[cron] built without `flows`; schedule trigger dropped");
                Ok(Attempt {
                    success: true,
                    output: format!("flow schedule tick emitted for flow {flow_id}"),
                    retryable: true,
                })
            })
            .await
    }
}

/// A host-owned system job (`system:<name>`): announced on the bus, then its
/// registered handler is awaited and its result is the run's result. Without a
/// handler the job is announce-only and `ok` on dispatch.
async fn run_system_job(job_id: &str, name: &str, run_id: &str) -> Attempt {
    tracing::info!(%job_id, job = %name, %run_id, "[cron] system job due — publishing CronSystemJobDue");
    BUS.publish(DomainEvent::CronSystemJobDue {
        job: name.to_string(),
    });
    let ctx = crate::cron::system_job_handlers::SystemJobContext {
        job_id: job_id.to_string(),
        name: name.to_string(),
    };
    let (success, output) = match crate::cron::system_job_handlers::dispatch(ctx).await {
        Some(Ok(())) => (true, format!("system job {name} completed")),
        Some(Err(error)) => (false, format!("system job {name} failed: {error}")),
        None => (true, format!("system job {name} dispatched")),
    };
    Attempt {
        success,
        output,
        retryable: true,
    }
}
