//! Published TinySecurity admission pins copied verbatim from v0.3.1/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.3.1",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.3.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.3.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "875e277852a4cdadf4a1505e1edd60da36a8df86a3e2e79f49605d5b5f371403",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.3.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "ecc200407e4d75d853720ba7dae1a4de7488f344c139940eba96e1826eec8393",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.3.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "d3c3d04068104a0763a2d71ed5bd6cfc98f60ce9f3cd5afbc954bc0198b89ca2",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.3.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "2f948239450dbd2998495cf97564d71461f4c361d3276cb01a01503f29a37fee",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.3.1-macos-26-arm64.tar.gz",
            sha256: "3776e0a93a9aa59b5725f668f901a2ceabd2ffd23a8cf9ad4c2a3597bab0af2d",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.3.1-macos-26-x86_64.tar.gz",
            sha256: "d24b50d4436ca21ad44cb5c4c557e51c1c03abfd2f49d2339f087daace027f58",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.3.1-macos-15-arm64.tar.gz",
            sha256: "cf3da67c586069caad82ddfb8ceae2fb11e65871d3c6af1dade1c98d142f76a0",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.3.1-macos-15-x86_64.tar.gz",
            sha256: "2d26b7dc46babb887205c33003f4b394708917bc50d4a4bc4f81064d3e8ddc06",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.3.1-windows-2025-x86_64.zip",
            sha256: "626f6a2505a0584a7d358450066cbf674196d334d8bae8b33161a9be0b5c08de",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.3.1-windows-2022-x86_64.zip",
            sha256: "3c96c8062d1509c5d1646b3b1d24d2eb555916a6e41f71fe41a8071411aaffc4",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.3.1-windows-11-arm64.zip",
            sha256: "7d3bfcec0856330d2358a06db4292067f8702351b88ece5c9b5101a5bbf7e748",
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
