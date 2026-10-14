//! Published TinySecurity admission pins copied verbatim from v0.6.0/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.6.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.6.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.6.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "5ecd66b557865b9f346ae2b829ee92041c2d968e16085422244692aaf0e1fe92",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.6.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "c757de2f0189e0913ae5e76d05e08b60615c09106916c649fb6035e0ba614e7d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.6.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "f2ab56c322f81cf3d894b86bbee2ca4a396d30860bed284ab10b7db2290e61ce",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.6.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "3df3d23e5a7a2da9d7ecc65f7a3dd633937d1bef928de93d6e51907f58a375c3",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.6.0-macos-26-arm64.tar.gz",
            sha256: "86a8ab9879a48211605e9e76810e2f0770d856c740e52eb8abc32489a1af3c00",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.6.0-macos-26-x86_64.tar.gz",
            sha256: "916435b7a8f16d121f0bfbb5f5c4d916178b7fa53416fcc6051c37be1778e217",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.6.0-macos-15-arm64.tar.gz",
            sha256: "538007a03ed1f17d2ed78d5f92b69276813a1283260091dcde7e836ef2ffd822",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.6.0-macos-15-x86_64.tar.gz",
            sha256: "63670774a429553ea6881761463dc858cec466f3cf24477c5eafde1de48966a2",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.6.0-windows-2025-x86_64.zip",
            sha256: "1d81badc0f5f44b5fa77cf82327bcdb0cab9b980d574062741ad97b2d4fb54c7",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.6.0-windows-2022-x86_64.zip",
            sha256: "69244319e14ccf6a43c4c424e8da4c75940fc381dabec8b501eb239502b33785",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.6.0-windows-11-arm64.zip",
            sha256: "0a6c771cec9fe983d0de5eca0843218ca52217e071eb9d67d868e5795cdf79b0",
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
