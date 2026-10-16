//! Published TinySecurity admission pins copied verbatim from v0.10.0/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.10.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.10.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.10.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "acac2cf60a0a70e3c68e815788cd160c43b1e8023df6803d2dbfba900957c376",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.10.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "6ac9ab50d48e32871291a72c9ebe84fce81013679b08be66dae89f1781a79128",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.10.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "f94630096c5ab9ea969f5d8d631850e95d25debf0af623f2f3cf60253152cde0",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.10.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "dd77b845c2c807d3b0863d0fe8dec1ff8b3b6847a1804fa6161cce9500b45be8",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.10.0-macos-26-arm64.tar.gz",
            sha256: "51f85f7996f5983f852a95414928a81c4f787a6f870137273c5d48511177c25e",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.10.0-macos-26-x86_64.tar.gz",
            sha256: "32ccf93cd30bd0e3486e06bacaa90060f2fa4f84478a371b6d2c9847967e9b18",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.10.0-macos-15-arm64.tar.gz",
            sha256: "ce74cdbbc8a4563dd678200985f570638f6b954d038fc4667f589ccc60470152",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.10.0-macos-15-x86_64.tar.gz",
            sha256: "9abebe44008894593b1ebcf61b98971ac6980af706e88fd1cdd386fda1c65307",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.10.0-windows-2025-x86_64.zip",
            sha256: "990717cbc02b45f88bd537594ef12fa653394f090491287f7e566d3d29144cc9",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.10.0-windows-2022-x86_64.zip",
            sha256: "b4d243d96c900eb30400fee9365ce8f0776a1f6c93e38dd03c46524862b9f33a",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.10.0-windows-11-arm64.zip",
            sha256: "791d167da7d999adee6c78958da765f5cbeca693f831603558bf3fa21e41034d",
        },
    ],
    load: LoadPolicy::Eager,
    source: SourceSpec {
        submodule: "tinysecurity",
        manifest: "crates/tinysecurity-module/Cargo.toml",
        package: "tinysecurity-module",
        lib_name: "tinysecurity_module",
        features: &[],
    },
};
