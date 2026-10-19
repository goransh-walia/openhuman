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
    version: "0.15.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.15.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.15.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "48bca9ec5875d7fd104b56725053bdf96f155929658d26f3e4201aeddcba2abe",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.15.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "a16272d27aa5e04cb5dd19bad8882c55691080f2276dcabbc54344a6513f3de9",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.15.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "dd33129f055e917d9030a78001dce78b3771b5e4c38e95ac46e5838e159a570c",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.15.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "5277a7df59def5e0c3dfc5dfe2017794ae3c52ee9cdd957d290664418ea01c86",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.15.0-macos-26-arm64.tar.gz",
            sha256: "ef0f368a46bb7f3d29bc06e656355e5b01b122dbc6e9208e8d615f276105d924",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.15.0-macos-26-x86_64.tar.gz",
            sha256: "91a4b33a2b46219dc596d8012e20114c81f33863e580eb3085e9cf472d3eba09",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.15.0-macos-15-arm64.tar.gz",
            sha256: "719be2707b0a8b9845cc2b3fb4289963f13f13cb34947183dfbcc68f3891885f",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.15.0-macos-15-x86_64.tar.gz",
            sha256: "cdb1dcbdf517022de7d4d1341b0cc30d153d68eed781918688609eaea9e77cdb",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.15.0-windows-2025-x86_64.zip",
            sha256: "a8d6446e9935811ecfe895b7d906958b233d44b4adb4d8bd9c3a10d688b92041",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.15.0-windows-2022-x86_64.zip",
            sha256: "362046d5a4dab3cf815b3f38e58d76324fa32b774b83de052c7c31dc57af82b5",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.15.0-windows-11-arm64.zip",
            sha256: "e7e9e19815e2accbb30ec96f5e12a86a24817d029f219ccf8d8a2b9f85cf02b3",
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
