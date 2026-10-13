use std::sync::atomic::Ordering;
use std::sync::Arc;

use tinymemory_api::MemoryEngine;

use super::*;
use crate::memory::layout_migration::test_host::{count, fact, FakeHost};
use crate::memory::layout_migration::{state, Phase};

fn never() -> Arc<dyn Fn() -> bool + Send + Sync> {
    Arc::new(|| false)
}

/// Waits (by yielding, not by the clock) for the workspace's run to end.
async fn settle(config: &Config) {
    for _ in 0..100_000 {
        if !is_running(config) {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("the run did not end");
}

#[tokio::test]
async fn scan_finds_legacy_memory_and_nothing_after_the_move() {
    let tmp = tempfile::tempdir().unwrap();
    let config = crate::memory::test_fixtures::config_in(&tmp);
    let host = Arc::new(FakeHost::with(3, &config).await);
    assert_eq!(
        scan(host.as_ref()).await.unwrap(),
        ScanView {
            needed: true,
            shared: false
        }
    );
    assert!(start(config.clone(), host.clone(), Trigger::Auto, never()));
    settle(&config).await;
    assert!(!scan(host.as_ref()).await.unwrap().needed);
    assert_eq!(status(&config).unwrap().state.phase, Phase::Cleaned);
}

#[tokio::test]
async fn nothing_to_move_shows_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let config = crate::memory::test_fixtures::config_in(&tmp);
    let host = FakeHost::with(0, &config).await;
    assert!(!scan(&host).await.unwrap().needed);
}

#[tokio::test]
async fn one_run_per_workspace_at_a_time() {
    let tmp = tempfile::tempdir().unwrap();
    let config = crate::memory::test_fixtures::config_in(&tmp);
    let host = Arc::new(FakeHost::with(200, &config).await);
    assert!(start(config.clone(), host.clone(), Trigger::Auto, never()));
    assert!(
        !start(config.clone(), host.clone(), Trigger::Auto, never()),
        "a second start while one runs starts nothing"
    );
    settle(&config).await;
    assert_eq!(count(host.tree.as_ref()).await, 200);
}

#[tokio::test]
async fn a_stopped_run_reads_as_interrupted_and_a_tick_resumes_it() {
    let tmp = tempfile::tempdir().unwrap();
    let config = crate::memory::test_fixtures::config_in(&tmp);
    let host = Arc::new(FakeHost::with(120, &config).await);
    let mut stopped = state::load(&config.workspace_dir).unwrap();
    stopped.phase = Phase::Copying;
    state::save(&config.workspace_dir, &stopped).unwrap();
    assert!(status(&config).unwrap().interrupted);

    assert!(tick(&config, host.clone(), never()));
    settle(&config).await;
    assert_eq!(status(&config).unwrap().state.phase, Phase::Cleaned);
    assert!(!tick(&config, host, never()), "nothing left to do");
}

#[tokio::test]
async fn nothing_starts_while_an_import_is_unfinished() {
    let tmp = tempfile::tempdir().unwrap();
    let config = crate::memory::test_fixtures::config_in(&tmp);
    let host = Arc::new(FakeHost::with(3, &config).await);
    let path = config
        .workspace_dir
        .join("memory")
        .join("import_state.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        r#"{"state":{"phase":"running","total":3,"imported":1}}"#,
    )
    .unwrap();
    assert!(crate::memory::import::in_progress(&config));
    assert!(!start(config.clone(), host.clone(), Trigger::Auto, never()));
    assert!(!tick(&config, host.clone(), never()));
    assert_eq!(count(host.tree.as_ref()).await, 0);

    std::fs::write(
        &path,
        r#"{"state":{"phase":"done","total":3,"imported":3}}"#,
    )
    .unwrap();
    assert!(start(config.clone(), host.clone(), Trigger::Auto, never()));
    settle(&config).await;
    assert_eq!(count(host.tree.as_ref()).await, 3);
}

#[tokio::test]
async fn a_tick_starts_nothing_while_background_work_is_paused() {
    let tmp = tempfile::tempdir().unwrap();
    let config = crate::memory::test_fixtures::config_in(&tmp);
    let host = Arc::new(FakeHost::with(3, &config).await);
    assert!(!tick(&config, host.clone(), Arc::new(|| true)));
    assert_eq!(count(host.tree.as_ref()).await, 0);
}

#[tokio::test]
async fn retry_puts_a_lost_copy_back_in_line() {
    let tmp = tempfile::tempdir().unwrap();
    let config = crate::memory::test_fixtures::config_in(&tmp);
    let host = Arc::new(FakeHost::with(4, &config).await);
    host.free.store(true, Ordering::SeqCst);
    assert!(start(config.clone(), host.clone(), Trigger::Auto, never()));
    settle(&config).await;
    assert_eq!(status(&config).unwrap().state.phase, Phase::Cleaned);

    // A legacy item the move did not reach (written after cleanup).
    host.legacy.store(fact("late")).await.unwrap();
    let state = retry(&config).unwrap();
    assert!(!state.caught_up && !state.cleaning);
    assert!(start(
        config.clone(),
        host.clone(),
        Trigger::Manual { takeover: false },
        never()
    ));
    settle(&config).await;
    assert_eq!(count(host.tree.as_ref()).await, 5);
    assert_eq!(count(host.legacy.as_ref()).await, 0);
}

#[tokio::test]
async fn a_damaged_state_file_surfaces_as_the_hosts_engine_error() {
    let tmp = tempfile::tempdir().unwrap();
    let config = crate::memory::test_fixtures::config_in(&tmp);
    std::fs::create_dir_all(config.workspace_dir.join("memory")).unwrap();
    std::fs::write(state::path(&config.workspace_dir), b"{ torn").unwrap();
    let error = status(&config).unwrap_err();
    assert_eq!(error.code(), "ENGINE");
    let host = Arc::new(FakeHost::with(1, &config).await);
    assert!(!tick(&config, host, never()), "a torn state starts nothing");
}
