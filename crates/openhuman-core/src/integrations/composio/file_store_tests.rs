use super::*;
use std::collections::BTreeMap;

fn config_in(dir: &tempfile::TempDir) -> Config {
    Config {
        workspace_dir: dir.path().to_path_buf(),
        ..Config::default()
    }
}

#[tokio::test]
async fn missing_value_loads_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let loaded: BTreeMap<String, u32> = load(&config_in(&dir), IDENTITIES_FILE).await.unwrap();
    assert!(loaded.is_empty());
}

#[tokio::test]
async fn save_then_load_round_trips_in_the_workspace_default_backend() {
    let dir = tempfile::tempdir().unwrap();
    let config = config_in(&dir);
    let mut value = BTreeMap::new();
    value.insert("gmail".to_string(), 3u32);
    save(&config, USER_SCOPES_FILE, &value).await.unwrap();
    let loaded: BTreeMap<String, u32> = load(&config, USER_SCOPES_FILE).await.unwrap();
    assert_eq!(loaded, value);
    // The other value is untouched, and nothing was written as a JSON file.
    let other: BTreeMap<String, u32> = load(&config, IDENTITIES_FILE).await.unwrap();
    assert!(other.is_empty());
    assert!(!dir.path().join("integrations").exists());
    assert!(dir.path().join("storage").exists());
}

#[tokio::test]
async fn update_applies_and_persists_a_change() {
    let dir = tempfile::tempdir().unwrap();
    let config = config_in(&dir);
    let bump = |v: &mut BTreeMap<String, u32>| {
        *v.entry("n".into()).or_default() += 1;
        ((), true)
    };
    update(&config, IDENTITIES_FILE, bump).await.unwrap();
    update(&config, IDENTITIES_FILE, bump).await.unwrap();
    let loaded: BTreeMap<String, u32> = load(&config, IDENTITIES_FILE).await.unwrap();
    assert_eq!(loaded["n"], 2);
}

#[tokio::test]
async fn a_legacy_state_file_is_imported_once_and_retired() {
    let dir = tempfile::tempdir().unwrap();
    let config = config_in(&dir);
    let legacy_dir = dir.path().join("integrations");
    std::fs::create_dir_all(&legacy_dir).unwrap();
    std::fs::write(legacy_dir.join(USER_SCOPES_FILE), r#"{"gmail":7}"#).unwrap();

    let loaded: BTreeMap<String, u32> = load(&config, USER_SCOPES_FILE).await.unwrap();
    assert_eq!(loaded["gmail"], 7);
    assert!(!legacy_dir.join(USER_SCOPES_FILE).exists());
    assert!(legacy_dir
        .join(format!("{USER_SCOPES_FILE}.migrated"))
        .exists());

    // A later edit is not clobbered by a re-import.
    update(
        &config,
        USER_SCOPES_FILE,
        |v: &mut BTreeMap<String, u32>| {
            v.insert("gmail".into(), 9);
            ((), true)
        },
    )
    .await
    .unwrap();
    let again: BTreeMap<String, u32> = load(&config, USER_SCOPES_FILE).await.unwrap();
    assert_eq!(again["gmail"], 9);
}
