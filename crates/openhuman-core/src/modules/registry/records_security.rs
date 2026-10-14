//! Published TinySecurity admission pins copied verbatim from v0.6.1/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.6.1",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.6.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.6.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "03b69da62a5f02f735a6069f0ce7b3410f34c05ed8d3b4474e187a1e6c492eda",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.6.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "2a807937270baeb9c519b3545c9e174f080f55100537f3b5e0eb2fbe9df9d901",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.6.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "2454b12620c2333bfedbfdb807c58989d0fba40229d83f18db25ae4b626845f2",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.6.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "69317239d9e51b813c591f99f79f9aa5f93315f7f5200c0fea4c53727c2edd07",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.6.1-macos-26-arm64.tar.gz",
            sha256: "242263136e18ea4213e37119344dbfe5acce298129f1468e80180a3bd9a94237",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.6.1-macos-26-x86_64.tar.gz",
            sha256: "de5a0bbb0a93a0554e0a8b194baca8743cb2c9baff244a2ada1cafdea694f7e0",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.6.1-macos-15-arm64.tar.gz",
            sha256: "ddfb114bd97aadfeef18a7ddccc73c5adf90ba870394580d9b7547c648bd61be",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.6.1-macos-15-x86_64.tar.gz",
            sha256: "766772270793c7d736b460cca41b865f2cf75445163699e7e0e6955d2134cd14",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.6.1-windows-2025-x86_64.zip",
            sha256: "0c8bc2523b09b26685fe474c2678c0c7d8f2e76a26c1cc9d557cf09637514ac8",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.6.1-windows-2022-x86_64.zip",
            sha256: "0fa36eb72d08cd4217e1539b3ea7996944f43c38a4d2edab399823f3b1a72181",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.6.1-windows-11-arm64.zip",
            sha256: "87d3fdd79a6860d37d3e807d31d1288f1e3e698456842e1229fbbca9aef405d9",
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
