use super::*;
use tinystoragedrivers::{MemoryStorage, Scope};

fn put_marker(stores: &ReboundSessionStores, backend: &Arc<dyn StorageBackend>) {
    // Opening an agent's stores declares its collections in the backend.
    let _ = stores.try_for_agent("probe").unwrap();
    backend.for_scope(&Scope::local()).unwrap();
}

#[test]
fn a_rebind_moves_new_agents_to_the_new_backend() {
    let first: Arc<dyn StorageBackend> = Arc::new(MemoryStorage::new());
    let second: Arc<dyn StorageBackend> = Arc::new(MemoryStorage::new());
    let stores = ReboundSessionStores::new(Arc::clone(&first), true).unwrap();
    put_marker(&stores, &first);
    let before = stores.destination_key();

    // Other tests' live stores in this process are rebound too.
    assert!(rebind_all(&second) >= 1);

    // The destination names the backend driver and a per-provider id; a
    // rebound store is a different provider underneath.
    assert_ne!(stores.destination_key(), before);
    stores.try_for_agent("probe").unwrap();
}

#[test]
fn handles_opened_before_a_rebind_keep_their_backend() {
    let first: Arc<dyn StorageBackend> = Arc::new(MemoryStorage::new());
    let second: Arc<dyn StorageBackend> = Arc::new(MemoryStorage::new());
    let stores = ReboundSessionStores::new(Arc::clone(&first), false).unwrap();
    let old = stores.try_for_agent("agent").unwrap();
    rebind_all(&second);
    let new = stores.try_for_agent("agent").unwrap();
    old.turn_states
        .put(&tinyagents_session::turn_state::TurnState::started(
            "t",
            "r",
            4,
            "2026-01-01T00:00:00Z",
        ))
        .ok();
    assert!(
        !Arc::ptr_eq(&old.turn_states, &new.turn_states),
        "a store opened after the rebind is not the one opened before it"
    );
}

#[test]
fn dropped_stores_are_not_rebound() {
    let backend: Arc<dyn StorageBackend> = Arc::new(MemoryStorage::new());
    let stores = ReboundSessionStores::new(Arc::clone(&backend), false).unwrap();
    drop(stores);
    let live_before = LIVE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .filter(|weak| weak.strong_count() > 0)
        .count();
    let rebound = rebind_all(&backend);
    assert!(rebound <= live_before);
}
