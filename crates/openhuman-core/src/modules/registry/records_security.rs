//! Published TinySecurity admission pins copied verbatim from v0.11.0/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.12.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.12.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.12.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "5f779d7fb7876120c0ea5270721bcd0d23c51f011abb3e9388c7ebc43541d1af",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.12.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "c0d3b38b37a490b6acded6a6a1d412ffbc5b600ae3c18e954386d5d88a8d0d01",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.12.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "814065b4c4c60ec7f7dc7f4f78a477eaab895f35da3b8a837eac238d1b32b88a",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.12.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "3faddf243c5c475735b0fc4f3b2a1087d6b67905cb2a2b810d2494223d26b31d",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.12.0-macos-26-arm64.tar.gz",
            sha256: "1c88163af8bef09fb4ff55175fb81263c6e2ecda90e1b2010ac2399693561d0a",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.12.0-macos-26-x86_64.tar.gz",
            sha256: "2318ed243797ad0bebd29aa77fe5ea667246696a09640ecc875048e6c18e4631",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.12.0-macos-15-arm64.tar.gz",
            sha256: "a99391e984cdb1afab88a2b4afc2a7061f101795ea62b1d5346ef4fba9dcb540",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.12.0-macos-15-x86_64.tar.gz",
            sha256: "5b6730a81763869158f12a0b05c9471dfab6f918642780df03ce3921f40cdaf6",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.12.0-windows-2025-x86_64.zip",
            sha256: "c0191cd625b9cccf5090e497772c0314e15fa7333af75ef31762fe4b19a8f040",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.12.0-windows-2022-x86_64.zip",
            sha256: "578126664e15632379f8601cca887950e4a5bc5f6177926096ec21de2316da8b",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.12.0-windows-11-arm64.zip",
            sha256: "faba50a25c94c84f5a46d6a62a2bea9a281d6c1c940a1623b4af96ee810529c3",
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
