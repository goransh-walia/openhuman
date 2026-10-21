//! Operator surface of the inbound event queues: depths and counts, the
//! dead-letter queue, and redriving one item.

use serde::Serialize;
use tinybus::module::ModuleClient;
use tinyeventqueues_bus::{DeadItem, StatsResponse};

use crate::config::Config;
use crate::core::Outcome;
use crate::modules::event_queues;

/// The most dead letters one `inbound.dead` call returns.
pub const MAX_DEAD_LIMIT: u32 = 500;

/// The dead letters `inbound.dead` returns.
#[derive(Debug, Clone, Serialize)]
pub struct DeadLetters {
    /// Parked items, oldest first.
    pub items: Vec<DeadItem>,
}

/// The result of `inbound.redrive`.
#[derive(Debug, Clone, Serialize)]
pub struct Redriven {
    /// The redriven item's id.
    pub id: String,
    /// `false` when the item was already gone.
    pub redriven: bool,
}

fn require(config: &Config) -> Result<(), String> {
    crate::core::runtime::components::require_config(config, super::settings::COMPONENT)
}

/// Queue depths and outcome counts.
///
/// # Errors
///
/// The component is off, or the module is unavailable or failed the call.
pub async fn stats(
    config: &Config,
    client: &ModuleClient,
) -> Result<Outcome<StatsResponse>, String> {
    require(config)?;
    let stats = event_queues::stats(client)
        .await
        .map_err(|error| format!("inbound.stats: {error}"))?;
    Ok(Outcome::single_log(stats, "inbound queue stats read"))
}

/// Up to `limit` dead letters (capped at [`MAX_DEAD_LIMIT`]).
///
/// # Errors
///
/// As [`stats`].
pub async fn dead(
    config: &Config,
    client: &ModuleClient,
    limit: Option<u32>,
) -> Result<Outcome<DeadLetters>, String> {
    require(config)?;
    let limit = limit.unwrap_or(50).min(MAX_DEAD_LIMIT);
    let items = event_queues::list_dead(client, limit)
        .await
        .map_err(|error| format!("inbound.dead: {error}"))?;
    let count = items.len();
    Ok(Outcome::single_log(
        DeadLetters { items },
        format!("{count} dead letter(s)"),
    ))
}

/// Put dead letter `id` back into the pipeline.
///
/// # Errors
///
/// As [`stats`]; an empty or unknown id is refused.
pub async fn redrive(
    config: &Config,
    client: &ModuleClient,
    id: &str,
) -> Result<Outcome<Redriven>, String> {
    require(config)?;
    if id.trim().is_empty() {
        return Err("inbound.redrive: id is required".to_owned());
    }
    let redriven = event_queues::redrive(client, id)
        .await
        .map_err(|error| format!("inbound.redrive: {error}"))?;
    tracing::info!(id = %id, redriven, "[inbound] redrive");
    Ok(Outcome::single_log(
        Redriven {
            id: id.to_owned(),
            redriven,
        },
        if redriven { "redriven" } else { "already gone" },
    ))
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
