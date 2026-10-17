//! Translation from product subagent outcomes to neutral orchestration records.

use super::*;

/// The host's in-flight subagent runs of one agent context.
pub(super) type HostInFlightMap = AsyncMutex<HashMap<HostLifecycleKey, Arc<HostInFlight>>>;

pub(super) fn host_outcome_to_neutral(
    outcome: SubagentRunOutcome,
    definition: &crate::agent::harness::host_definition::AgentDefinition,
    options: &SubagentRunOptions,
) -> SubagentOutcome {
    let status = match outcome.status {
        SubagentRunStatus::Completed => SubagentOutcomeKind::Completed,
        SubagentRunStatus::AwaitingUser { question, .. } => {
            SubagentOutcomeKind::AwaitingInput(SubagentPause {
                reason: question,
                resume: SubagentResume {
                    history: outcome
                        .final_history
                        .iter()
                        .map(crate::agent::message_convert::chat_message_to_message)
                        .collect(),
                    metadata: std::collections::BTreeMap::from_iter([
                        ("agent_id".into(), definition.id.clone()),
                        (
                            "worker_thread_id".into(),
                            options.worker_thread_id.clone().unwrap_or_default(),
                        ),
                        (
                            "skill_filter_override".into(),
                            options.skill_filter_override.clone().unwrap_or_default(),
                        ),
                        (
                            "model_override".into(),
                            options.model_override.clone().unwrap_or_default(),
                        ),
                    ]),
                    ..SubagentResume::default()
                },
            })
        }
        SubagentRunStatus::Incomplete { reason } => {
            SubagentOutcomeKind::Incomplete(SubagentIncomplete::new(reason))
        }
        SubagentRunStatus::Cancelled => SubagentOutcomeKind::Cancelled,
    };
    SubagentOutcome {
        task_id: outcome.task_id,
        output: outcome.output,
        history: outcome
            .final_history
            .iter()
            .map(crate::agent::message_convert::chat_message_to_message)
            .collect(),
        status,
        usage: UsageTotals {
            calls: outcome.iterations as u64,
            usage: Usage {
                input_tokens: outcome.usage.input_tokens,
                output_tokens: outcome.usage.output_tokens,
                total_tokens: outcome
                    .usage
                    .input_tokens
                    .saturating_add(outcome.usage.output_tokens),
                cache_read_tokens: outcome.usage.cached_input_tokens,
                // An unknown cost crosses as no charge at all, never as $0.
                charged_amount: outcome
                    .usage
                    .cost()
                    .usd()
                    .map(|usd| ChargedAmount::usd_micros((usd * 1_000_000.0).round() as i64)),
                ..Usage::default()
            },
        },
        artifacts: outcome
            .artifact_paths
            .into_iter()
            .map(|id| ArtifactReference {
                id,
                ..ArtifactReference::default()
            })
            .collect(),
        ..SubagentOutcome::cancelled(String::new())
    }
}

pub(super) fn observer_cancelled_outcome(
    task_key: &SubagentTaskKey,
    agent_id: &str,
) -> SubagentRunOutcome {
    SubagentRunOutcome {
        task_id: task_key.task_id.clone(),
        agent_id: agent_id.to_owned(),
        output: String::new(),
        iterations: 0,
        elapsed: std::time::Duration::default(),
        mode: super::super::SubagentMode::Typed,
        status: SubagentRunStatus::Cancelled,
        final_history: Vec::new(),
        usage: SubagentUsage::default(),
        artifact_paths: Vec::new(),
        persistence_disposition:
            tinyagents_orchestration::subagent::SubagentPersistenceDisposition::ObserverCancelled,
    }
}

pub(super) fn host_observer_outcome(mut outcome: SubagentRunOutcome) -> SubagentRunOutcome {
    use tinyagents_orchestration::subagent::SubagentPersistenceDisposition as Disposition;

    outcome.persistence_disposition = match outcome.persistence_disposition {
        Disposition::PauseCommitted | Disposition::PauseReplaced | Disposition::PauseExisting => {
            Disposition::PauseExisting
        }
        Disposition::TerminalInserted | Disposition::TerminalExisting => {
            Disposition::TerminalExisting
        }
        Disposition::ObserverCancelled => Disposition::ObserverCancelled,
    };
    outcome
}
