//! Registry record for the `tinyeventqueues` module.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

/// The `tinyeventqueues` module: durable inbound event queues. It persists,
/// deduplicates, filters and paces inbound events (Composio triggers, agent
/// webhooks) on the host's storage, loaded on first ingest, and calls the
/// host back (`Deliver`, `ClassifySalience`) to run triage.
pub(crate) const TINYEVENTQUEUES: ModuleRecord = ModuleRecord {
    id: "tinyeventqueues",
    description: "Durable inbound event queues: dedupe, filtering, pacing and retry before triage",
    bus_name: tinyeventqueues_bus::INTERFACE,
    object_path: tinyeventqueues_bus::OBJECT_PATH,
    version: "0.1.2",
    release_url: "https://github.com/tinyhumansai/tinyeventqueues/releases/tag/v0.1.2",
    assets: &[
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyeventqueues-0.1.2-macos-15-arm64.tar.gz",
            sha256: "fa4c7ef8b9cbd11737f14e7e97a93066f251bd8a1604b0b62db1aff9879da3b1",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyeventqueues-0.1.2-macos-15-x86_64.tar.gz",
            sha256: "f185405974dc0f535b73110cc03d43b3a3a79c8360ab3464872d3a578b8ff14e",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyeventqueues-0.1.2-macos-26-arm64.tar.gz",
            sha256: "a5a1f77642fbdecb47c947181a69fd7800dc5af3b7f65ee727a8136a74798da9",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyeventqueues-0.1.2-macos-26-x86_64.tar.gz",
            sha256: "9545786aae74159d59e151a418a7cc905fa74c2aa8bbb6f709ad5dfa6fc37836",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyeventqueues-0.1.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "c838389f4cdce63bf9b4da51d95847a47a7c9e7f17e2020b9c2f87b84607255f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyeventqueues-0.1.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "2ff9b8883af8c92cce16bed5128c06c3e784096c6d8d3da1767b15772a3aaa99",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyeventqueues-0.1.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "fe2bf48c2e2f0147c71bb8e3353fb5c1c3c540c3304c0064929ad328f11dc9b6",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyeventqueues-0.1.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "eb2a4cab5699e5d8a66737183948e67da767b4ee8f524f783f148297a2f4ed28",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyeventqueues-0.1.2-windows-11-arm64.zip",
            sha256: "b097798844a063d721f7312c5fe0744e540715dc54717a7c279527beb7ee8edf",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyeventqueues-0.1.2-windows-2022-x86_64.zip",
            sha256: "4f000b4205e604236f5f98021eddca5f967ae426e2eabe0e99b51536d2d8e3e4",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyeventqueues-0.1.2-windows-2025-x86_64.zip",
            sha256: "46e9ac7d7a955d641d20d72ffa387d07b066cd3308fc3e09d061dcaf02fcc32e",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinyeventqueues",
        manifest: "crates/tinyeventqueues/Cargo.toml",
        package: "tinyeventqueues",
        lib_name: "tinyeventqueues",
        features: &[],
    },
};
