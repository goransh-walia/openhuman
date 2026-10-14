//! Published TinySecurity admission pins copied verbatim from v0.5.0/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.5.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.5.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.5.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "0af1cad99f9d4f9363eece9831603ef464b1468ea4d7e327aff2183cb5ad3330",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.5.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "019cf7257de74865a54da2c19d9616d6c1e95bb8a991c56f1a5b4c2282d02d4a",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.5.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "7c3c087435753cff65f9d26509fe5fcf3d72947094d45bbeaf38565974bdcd68",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.5.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "0190a7329ae2887202033f94c0d4c81ea5a70004b84112f8c5f557a950621cfb",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.5.0-macos-26-arm64.tar.gz",
            sha256: "9df106d8fa313929dfee88fadeb0c0bfa69d51e9ee63fcf8f63f5f5aa56032f7",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.5.0-macos-26-x86_64.tar.gz",
            sha256: "2fd1d394d784e59bbdd5302aa2a2e771a177f38b9d26d33d46f4b3e71fa8765f",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.5.0-macos-15-arm64.tar.gz",
            sha256: "04af476ae3ab472ad5e4ba3896ed300c0a5d8dc6c576496733a5f29c8080deb3",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.5.0-macos-15-x86_64.tar.gz",
            sha256: "5aa7a01232ee1aef8777df27ac0be492335751df784fdbacc6db2b2865255a8f",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.5.0-windows-2025-x86_64.zip",
            sha256: "fb60522475ff32b5008c7135ca55f33b7995be8ecdf13cbdda8a6f5f9ec84795",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.5.0-windows-2022-x86_64.zip",
            sha256: "e1e1f7934f5fae463d15cc3b3b215836d69fee9bb4117c0ac2ed530abaf8be23",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.5.0-windows-11-arm64.zip",
            sha256: "f88bbc4c13aefc92ebc0588e6806261df147edf690e364446de932de53f15a58",
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
