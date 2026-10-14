//! Periodic reclamation of expired blobs.
//!
//! A blob written with a lifetime reads as absent once it lapses, but its
//! bytes stay in the backend until something sweeps them
//! (`StorageBackend::sweep_expired_blobs`, the trait-level hook the
//! tinystoragedrivers drivers implement). The host owns the schedule: this
//! module visits every storage scope background work covers
//! ([`super::agents::for_each_scope`]) on a fixed tick and asks the installed
//! backend to reclaim that scope's lapsed blobs.
//!
//! It is best-effort maintenance. A scope that fails to sweep is logged and
//! retried on the next tick; nothing here blocks boot or a request.

use std::sync::Arc;
use std::time::Duration;

use tinystoragedrivers::{Scope, StorageBackend};

/// How often the sweeper wakes. Lapsed blobs are already invisible to readers,
/// so this only bounds how long their bytes linger.
pub const SWEEP_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Delay before the first sweep, so boot work is not competing with it.
pub const FIRST_SWEEP_DELAY: Duration = Duration::from_secs(5 * 60);

/// Reclaims the lapsed blobs of `scope`, returning how many went. A failure
/// is logged and counts as zero.
pub async fn sweep_scope(backend: &dyn StorageBackend, scope: &Scope) -> u64 {
    match backend.sweep_expired_blobs(scope).await {
        Ok(removed) => {
            if removed > 0 {
                tracing::debug!(
                    scope = scope.as_str(),
                    removed,
                    "[storage::blob_sweep] reclaimed expired blobs"
                );
            }
            removed
        }
        Err(error) => {
            tracing::warn!(
                scope = scope.as_str(),
                %error,
                "[storage::blob_sweep] sweep failed; retrying on the next tick"
            );
            0
        }
    }
}

/// One pass over every scope background work covers (`local` outside SaaS
/// mode, then each known agent), against `backend`. Returns the total number
/// of blobs reclaimed.
pub async fn sweep_all(backend: Arc<dyn StorageBackend>) -> u64 {
    super::agents::for_each_scope("blob sweep", || {
        let backend = Arc::clone(&backend);
        async move {
            match super::current_scope() {
                Ok(scope) => {
                    // Modules keep their expiring blobs in their own scopes
                    // derived from the acting scope (TinyJuice's CCR TTL).
                    let mut removed = sweep_scope(backend.as_ref(), &scope).await;
                    for module_scope in crate::modules::expiring_blob_module_scopes() {
                        if module_scope != scope {
                            removed += sweep_scope(backend.as_ref(), &module_scope).await;
                        }
                    }
                    removed
                }
                Err(error) => {
                    tracing::debug!(%error, "[storage::blob_sweep] no scope for this visit; skipped");
                    0
                }
            }
        }
    })
    .await
    .into_iter()
    .map(|(_, removed)| removed)
    .sum()
}

/// [`sweep_all`] against the installed backend; zero when none is installed.
pub async fn sweep_installed() -> u64 {
    match super::installed() {
        Some(backend) => sweep_all(backend).await,
        None => 0,
    }
}

/// Spawns the sweeper: after `first_delay`, then every `interval`, until the
/// task is aborted. Missed ticks are skipped, not replayed.
pub fn spawn(first_delay: Duration, interval: Duration) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        tracing::debug!(
            first_delay_secs = first_delay.as_secs(),
            interval_secs = interval.as_secs(),
            "[storage::blob_sweep] scheduled"
        );
        tokio::time::sleep(first_delay).await;
        let mut tick = tokio::time::interval(interval);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            let removed = sweep_installed().await;
            tracing::trace!(removed, "[storage::blob_sweep] tick complete");
        }
    })
}

#[cfg(test)]
#[path = "blob_sweep_tests.rs"]
mod tests;
