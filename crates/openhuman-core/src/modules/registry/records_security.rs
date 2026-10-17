//! Published TinySecurity admission pins copied verbatim from v0.13.0/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.14.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.14.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.14.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "6fcb02755595857953ff226c0baccc3a5ab335ae9116f9f2d2974fce81017f17",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.14.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "94e7c9b7e59c515e6ee0ee4e790f6330c58f862b99f428d32c4a3a3b38cf81e4",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.14.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "ed51a3b74076f350cda322eafc8cdae3a5d195ebd1a3878712a3b378ef6aa226",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.14.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "2d231a243c651c6163def9c6bf08e2bdc74ee306a8efe0f9a35e2d74290b184a",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.14.0-macos-26-arm64.tar.gz",
            sha256: "9cfefd44f1a13d15a25f4015ad4d99771db2a59882c2ed40a6d98ac949917e2c",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.14.0-macos-26-x86_64.tar.gz",
            sha256: "279459dd043a21ea6c48be3bd17c1c284a142ed35d1a6636489ffb8fdc6c556d",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.14.0-macos-15-arm64.tar.gz",
            sha256: "29fe56714e3e1130bfd826920d45c1267ad5d978a4c3c706ec7282a3f126dc0a",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.14.0-macos-15-x86_64.tar.gz",
            sha256: "0851994d9e601e794ebc383c7bf35f7e2a35046bb53fdec22e8a198c052b2807",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.14.0-windows-2025-x86_64.zip",
            sha256: "9d0b55bc5b74023a5c72a75ede3b5c670b519844188ba32014f66784179601fb",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.14.0-windows-2022-x86_64.zip",
            sha256: "40b2b83c60262a9c05bfb264dd878f4d36d31edc3f51dce469448aaa67c55133",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.14.0-windows-11-arm64.zip",
            sha256: "7e14fd866ec809749d59b83799ee748e761dbd32aac74b4568a460f5968b676e",
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
