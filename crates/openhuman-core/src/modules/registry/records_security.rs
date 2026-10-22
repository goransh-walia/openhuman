//! Published TinySecurity admission pins copied verbatim from v0.16.0/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.16.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.16.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.16.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "a7a6d77dbb1d944d5cff05df9bdae7afca3400cea2f143d9582211ce119993ff",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.16.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "fe7b37a3ea3c44815577fe67474a4491f85759d1907954c39a838081753ee636",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.16.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "cebd42e4e67bd926be48c9b0a858c56c334ce5a35decdcbfb368d75ef134f0e1",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.16.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "d6157ff83beee777c12e2cce14af3a71cdfd9780def26e2ec43dee592fc921c4",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.16.0-macos-26-arm64.tar.gz",
            sha256: "727ea3f01c6f31fde80d8686256fc23f5d1f2eb6880abcda1de3d89002f90f15",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.16.0-macos-26-x86_64.tar.gz",
            sha256: "21c49c77bb9676cbd7bbac8b67cd383c010df9a76421aafdf6ac07accead06b1",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.16.0-macos-15-arm64.tar.gz",
            sha256: "f7f15099d1a116465338420c2a6a2da9e20c4d008d1bf5b2b88a6852e7e0758f",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.16.0-macos-15-x86_64.tar.gz",
            sha256: "dc3dd1657653e83c4592db46af4e8f412663bc4b3ead230675e51ab71c8c065c",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.16.0-windows-2025-x86_64.zip",
            sha256: "3206ec937fd74f2fc32c78012900388a738f29c2dc134ef9c9c094dce547ed8b",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.16.0-windows-2022-x86_64.zip",
            sha256: "18ace3e79ef6415dc8f1fa2c0afe1336254f0f93f7c0bb8cfe725133353b676f",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.16.0-windows-11-arm64.zip",
            sha256: "e3067f3475990879e1145ff09ca55a463bd0e09f68965b936a4842b1b49296a2",
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
