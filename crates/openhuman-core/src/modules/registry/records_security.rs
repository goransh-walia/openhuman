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
    version: "0.11.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.11.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.11.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "34942ad032fe2f2185f4fae61903bf9816b33354c8b5eb3cfb9b862c67f8f9a0",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.11.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "94e6200db2cd72391293073dbeedc461daf3f2d3331d34aa8337f5344921daea",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.11.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "9548d08a2f3e3c92c3e04c6ee6f144dea9962f14f1294d22219d8db3647f72d8",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.11.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "47a2a72d8b5e4f6809e7a1c10e3cc865c67d1cdcb5cf01f9c403ce80b75f216b",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.11.0-macos-26-arm64.tar.gz",
            sha256: "e25748c0411cfc83673166f1743008ab46808eeac917b7574a09097fede0c359",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.11.0-macos-26-x86_64.tar.gz",
            sha256: "bda3931e453c27d43fa64a57d38e4a16494010d997227feaaad6a49cc1d97833",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.11.0-macos-15-arm64.tar.gz",
            sha256: "107a102e2e6f39d4a2e027bcf99d18a76ba8331da6658eec4f0d0eb15ce13069",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.11.0-macos-15-x86_64.tar.gz",
            sha256: "fbec157d8d2e7238e9f0087fd20ad8d31526b983967a6ac7570d7ba144ce7bcf",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.11.0-windows-2025-x86_64.zip",
            sha256: "63abd61675ff0e5bfd0f8a24f56f644f35487cd52a1d5895bbaaaf65f5697238",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.11.0-windows-2022-x86_64.zip",
            sha256: "46e9132ea9825a4513c1c6aade5c0940badf308d49268e6766b79f04cf8404b3",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.11.0-windows-11-arm64.zip",
            sha256: "a21a66a4d291ce81b93ba076e0020f01c1ba592f96c09be14595616773da82f9",
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
