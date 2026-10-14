//! Published TinySecurity admission pins copied verbatim from v0.4.0/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.4.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.4.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.4.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "b7c31a75c83676ca6003de37011a1c1ca9cd22480e8825393c206ab57e16688c",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.4.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "61327ab5f0affc16d46888bad90141c83a825fc912972c4ac603894cdaa0d76b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.4.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "19da67cabe53c67e069348455e1bbaf9f5db823c34ea231063079273bb98d841",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.4.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "deed4589f3bc8e14a44782ef9a8438b3d577fedfad5507ea61408c145ae2b73e",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.4.0-macos-26-arm64.tar.gz",
            sha256: "504a951c16890052ccade8c61c687284c671c6bbd482553a38391c79c3fdf736",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.4.0-macos-26-x86_64.tar.gz",
            sha256: "d8057604c51813645b1092bd4af1dd9eaba90466a572aecf76b2c0d42010e8d4",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.4.0-macos-15-arm64.tar.gz",
            sha256: "abbf8c381d3b58b2a8eef26b6e7c44b2d3b92c68597be41f3ed40ae90b1a083b",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.4.0-macos-15-x86_64.tar.gz",
            sha256: "b966b8009666cc11aecd977365eed422b40a25945caa53a84e17d12a9b986815",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.4.0-windows-2025-x86_64.zip",
            sha256: "555752b7384dc817d8c165eee7b62e984f1ce3ce075b6c1f942811984419697e",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.4.0-windows-2022-x86_64.zip",
            sha256: "d5200def4c650a9d783468c07f2e5e606fdce5f076ced42478c1bd96d90a7b3c",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.4.0-windows-11-arm64.zip",
            sha256: "e91b15c9d904d684485b536e0fabad6e5edfcfd3ba4a7e9cf3c3247deeef75d4",
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
