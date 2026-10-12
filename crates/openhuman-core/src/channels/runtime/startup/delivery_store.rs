//! Durable host-owned storage for TinyChannels delivery callbacks.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use tinychannels_bus::module_delivery::{
    DeliveryQueueEntry, DeliveryTerminalResult, DELIVERY_PENDING_CAPACITY, DELIVERY_RECORD_BYTES,
    DELIVERY_RESULT_CAPACITY, DELIVERY_RESULT_TTL_SECONDS,
};
use tokio::sync::Mutex;

const FILE_NAME: &str = "channel-delivery.json";
const MAX_STORE_BYTES: u64 = (DELIVERY_PENDING_CAPACITY * DELIVERY_RECORD_BYTES * 2) as u64
    + (DELIVERY_RESULT_CAPACITY * DELIVERY_RECORD_BYTES) as u64;

type Locks = HashMap<PathBuf, Arc<Mutex<()>>>;
static WORKSPACE_LOCKS: OnceLock<StdMutex<Locks>> = OnceLock::new();

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct DeliveryState {
    pending: BTreeMap<String, DeliveryQueueEntry>,
    failed: BTreeMap<String, DeliveryQueueEntry>,
    results: BTreeMap<String, DeliveryTerminalResult>,
}

/// Shared store keyed by the captured workspace, so concurrent channel
/// sessions serialize updates to the same durable queue.
#[derive(Clone)]
pub(super) struct ChannelDeliveryStore {
    path: PathBuf,
    gate: Arc<Mutex<()>>,
}

impl ChannelDeliveryStore {
    pub(super) fn for_workspace(workspace: &Path) -> Self {
        let path = workspace.join("channels").join(FILE_NAME);
        let locks = WORKSPACE_LOCKS.get_or_init(|| StdMutex::new(HashMap::new()));
        let gate = locks
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .entry(path.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        Self { path, gate }
    }

    async fn locked_state(
        &self,
    ) -> anyhow::Result<(tokio::sync::OwnedMutexGuard<()>, DeliveryState)> {
        let guard = Arc::clone(&self.gate).lock_owned().await;
        let state = self.read_state().await?;
        Ok((guard, state))
    }

    async fn read_state(&self) -> anyhow::Result<DeliveryState> {
        let bytes = match tokio::fs::read(&self.path).await {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(DeliveryState::default());
            }
            Err(error) => return Err(error.into()),
        };
        anyhow::ensure!(
            bytes.len() as u64 <= MAX_STORE_BYTES,
            "channel delivery store is oversized"
        );
        let state: DeliveryState = serde_json::from_slice(&bytes)?;
        validate_state(&state)?;
        Ok(state)
    }

    async fn write_state(&self, state: DeliveryState) -> anyhow::Result<()> {
        validate_state(&state)?;
        let bytes = serde_json::to_vec(&state)?;
        anyhow::ensure!(
            bytes.len() as u64 <= MAX_STORE_BYTES,
            "channel delivery store is oversized"
        );
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || {
            crate::security::secret_storage::file_store::write_atomic(&path, &bytes)
                .map_err(anyhow::Error::from)
        })
        .await??;
        Ok(())
    }

    pub(super) async fn save_pending(&self, entry: DeliveryQueueEntry) -> anyhow::Result<()> {
        validate_entry(&entry)?;
        let (_guard, mut state) = self.locked_state().await?;
        if let Some(existing) = state.pending.get(&entry.id) {
            anyhow::ensure!(
                existing == &entry,
                "delivery id conflicts with pending entry"
            );
            return Ok(());
        }
        anyhow::ensure!(
            !state.failed.contains_key(&entry.id) && !state.results.contains_key(&entry.id),
            "delivery id already has a terminal result"
        );
        anyhow::ensure!(
            state.pending.len() < DELIVERY_PENDING_CAPACITY,
            "delivery queue is full"
        );
        state.pending.insert(entry.id.clone(), entry);
        self.write_state(state).await
    }

    pub(super) async fn load_pending(
        &self,
        id: String,
    ) -> anyhow::Result<Option<DeliveryQueueEntry>> {
        validate_id(&id)?;
        let (_guard, state) = self.locked_state().await?;
        Ok(state.pending.get(&id).cloned())
    }

    pub(super) async fn load_pending_all(&self) -> anyhow::Result<Vec<DeliveryQueueEntry>> {
        let (_guard, state) = self.locked_state().await?;
        let mut entries: Vec<_> = state.pending.into_values().collect();
        entries.sort_by(|left, right| {
            left.enqueued_at_unix_ms
                .cmp(&right.enqueued_at_unix_ms)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(entries)
    }

    pub(super) async fn remove_pending(&self, id: String) -> anyhow::Result<()> {
        validate_id(&id)?;
        let (_guard, mut state) = self.locked_state().await?;
        if state.pending.remove(&id).is_some() {
            self.write_state(state).await?;
        }
        Ok(())
    }

    pub(super) async fn save_failed(&self, entry: DeliveryQueueEntry) -> anyhow::Result<()> {
        validate_entry(&entry)?;
        let (_guard, mut state) = self.locked_state().await?;
        if let Some(existing) = state.failed.get(&entry.id) {
            anyhow::ensure!(
                existing == &entry,
                "delivery id conflicts with failed entry"
            );
            return Ok(());
        }
        anyhow::ensure!(
            state.failed.len() < DELIVERY_PENDING_CAPACITY,
            "failed delivery store is full"
        );
        state.failed.insert(entry.id.clone(), entry);
        self.write_state(state).await
    }

    pub(super) async fn load_failed(
        &self,
        id: String,
    ) -> anyhow::Result<Option<DeliveryQueueEntry>> {
        validate_id(&id)?;
        let (_guard, state) = self.locked_state().await?;
        Ok(state.failed.get(&id).cloned())
    }

    pub(super) async fn save_result(&self, result: DeliveryTerminalResult) -> anyhow::Result<()> {
        validate_id(&result.id)?;
        anyhow::ensure!(
            !result.intent_fingerprint.is_empty() && result.intent_fingerprint.len() <= 128,
            "delivery fingerprint is invalid"
        );
        anyhow::ensure!(
            serde_json::to_vec(&result).is_ok_and(|bytes| bytes.len() <= DELIVERY_RECORD_BYTES),
            "delivery result exceeds its record limit"
        );
        let (_guard, mut state) = self.locked_state().await?;
        prune_results(&mut state, now_unix_ms());
        if let Some(existing) = state.results.get(&result.id) {
            anyhow::ensure!(
                existing == &result,
                "delivery id conflicts with terminal result"
            );
            return Ok(());
        }
        anyhow::ensure!(
            state.results.len() < DELIVERY_RESULT_CAPACITY,
            "delivery result replay store is full"
        );
        state.results.insert(result.id.clone(), result);
        self.write_state(state).await
    }

    pub(super) async fn load_result(
        &self,
        id: String,
    ) -> anyhow::Result<Option<DeliveryTerminalResult>> {
        validate_id(&id)?;
        let (_guard, mut state) = self.locked_state().await?;
        prune_results(&mut state, now_unix_ms());
        let result = state.results.get(&id).cloned();
        if result.is_none() {
            self.write_state(state).await?;
        }
        Ok(result)
    }
}

fn validate_state(state: &DeliveryState) -> anyhow::Result<()> {
    anyhow::ensure!(
        state.pending.len() <= DELIVERY_PENDING_CAPACITY,
        "pending delivery store exceeds capacity"
    );
    anyhow::ensure!(
        state.failed.len() <= DELIVERY_PENDING_CAPACITY,
        "failed delivery store exceeds capacity"
    );
    anyhow::ensure!(
        state.results.len() <= DELIVERY_RESULT_CAPACITY,
        "delivery result store exceeds capacity"
    );
    for (id, entry) in state.pending.iter().chain(state.failed.iter()) {
        anyhow::ensure!(id == &entry.id, "delivery store key does not match record");
        validate_entry(entry)?;
    }
    for (id, result) in &state.results {
        validate_id(id)?;
        anyhow::ensure!(
            id == &result.id,
            "delivery result key does not match record"
        );
    }
    Ok(())
}

fn validate_entry(entry: &DeliveryQueueEntry) -> anyhow::Result<()> {
    validate_id(&entry.id)?;
    anyhow::ensure!(
        entry.intent.idempotency_key == entry.id,
        "delivery id does not match intent key"
    );
    anyhow::ensure!(
        serde_json::to_vec(entry).is_ok_and(|bytes| bytes.len() <= DELIVERY_RECORD_BYTES),
        "delivery record exceeds its limit"
    );
    Ok(())
}

fn validate_id(id: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!id.is_empty() && id.len() <= 256, "delivery id is invalid");
    Ok(())
}

fn prune_results(state: &mut DeliveryState, now_ms: u64) {
    let ttl_ms = DELIVERY_RESULT_TTL_SECONDS.saturating_mul(1000);
    state
        .results
        .retain(|_, result| result.completed_at_unix_ms.saturating_add(ttl_ms) > now_ms);
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

#[cfg(test)]
#[path = "delivery_store_tests.rs"]
mod tests;
