use super::*;

#[test]
fn every_linked_crate_exposes_its_valid_collection_declarations() {
    let owners = collection_owners().expect("owner collection declarations are valid");
    let mut crates = std::collections::HashSet::new();
    let mut names = std::collections::HashSet::new();
    for owner in &owners {
        assert!(
            crates.insert(owner.crate_name),
            "duplicate crate {}",
            owner.crate_name
        );
        for spec in &owner.collections {
            spec.validate().expect("owner collection is valid");
            assert!(
                names.insert((owner.crate_name, spec.name.as_str())),
                "duplicate {} collection in {}",
                spec.name,
                owner.crate_name
            );
        }
    }
    for expected in [
        ("tinyagents-harness", "harness.run_status"),
        ("tinyagents-session", "conversations.threads"),
        ("openhuman-core", "approvals.requests"),
        ("openhuman-core", "devices.paired"),
        ("openhuman-core", "notifications.integration"),
        ("openhuman-core", "task_sources.sources"),
        ("openhuman-core", "artifacts.records"),
        ("openhuman-core", "cost.records"),
        ("openhuman-core", "config.documents"),
        ("openhuman-core", "profiles.registry"),
        ("openhuman-core", "threads.transcript_keys"),
        ("openhuman-core", "subagents.sessions"),
    ] {
        assert!(names.contains(&expected), "{expected:?} missing: {names:?}");
    }
}

#[test]
fn module_collections_are_limited_to_the_named_owner() {
    let mcp = module_collections("mcp");
    assert!(mcp.iter().any(|spec| spec.name == "mcp.servers"));
    assert!(module_collections("no-such-owner").is_empty());
}

#[test]
fn tinyjuice_ccr_blobs_stay_inside_the_juice_owner_namespace() {
    assert_eq!(module_items("juice").blobs, ["juice/ccr/"]);
}

#[cfg(feature = "modules")]
#[tokio::test]
async fn tinyjuice_legacy_ccr_blobs_migrate_with_the_owner_contract() {
    use tinystoragedrivers::schema::{MigrateOptions, Registry};
    use tinystoragedrivers::{MemoryStorage, Scope, StorageBackend};

    let backend = MemoryStorage::new();
    let scoped = backend.for_scope(&Scope::local()).unwrap();
    let blobs = scoped.blobs();
    blobs
        .put(
            "tinyjuice_ccr/0123456789abcdef",
            b"original text".to_vec(),
            Some("text/plain; charset=utf-8"),
        )
        .await
        .unwrap();
    blobs
        .put("tinyjuice_ccr/fedcba9876543210", b"two".to_vec(), None)
        .await
        .unwrap();
    blobs
        .put("other/keep", b"keep".to_vec(), None)
        .await
        .unwrap();

    let registry = Registry::compose([tinyjuice_storage_schema()]).unwrap();
    let report = registry
        .migrate(&scoped, &MigrateOptions::new())
        .await
        .unwrap();
    assert_eq!(report.ran[0].written, 2);
    assert!(blobs
        .list(tinyjuice_bus::storage::LEGACY_CCR_PREFIX)
        .await
        .unwrap()
        .is_empty());
    let moved = blobs
        .get("juice/ccr/0123456789abcdef")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(moved.bytes, b"original text");
    assert_eq!(
        moved.meta.content_type.as_deref(),
        Some("text/plain; charset=utf-8")
    );
    let moved = blobs
        .get("juice/ccr/fedcba9876543210")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(moved.meta.content_type, None);
    assert!(blobs.get("other/keep").await.unwrap().is_some());

    let again = registry
        .migrate(&scoped, &MigrateOptions::new())
        .await
        .unwrap();
    assert!(again.ran.is_empty());
}
