//! Published TinySecurity admission pins copied verbatim from v0.8.0/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.9.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.9.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.9.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "76ff39c14b3d03ae6192b19d8ddcebc1eff7db1ae7d204b58a4b924388d63810",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.9.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "9fc105d4d1826496729770afe77e9f257dd793ce7dc9cb7c36a1c7c8d8a1941b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.9.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "e0076343ce2aeca41cd52c5a6f0a6d15a00cc7b0f475fcb25b1bc27d2147f727",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.9.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "5591be6782acc43e86b8be1fcb6e9c0e6be27a88d2160e8204acdeccb1d1877d",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.9.0-macos-26-arm64.tar.gz",
            sha256: "b2eb7e3ef69a02377393c37a33cd5ceca136150a82067d28312d47da4883bfe5",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.9.0-macos-26-x86_64.tar.gz",
            sha256: "271f7d39d28ca4018dc303e86f39a13a83715a1bb17f93205ad36ee5212f07c5",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.9.0-macos-15-arm64.tar.gz",
            sha256: "7801ec379a97d62b36b4f1f8212e59db0aa62a462b941cc89ff8ffc0d374088e",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.9.0-macos-15-x86_64.tar.gz",
            sha256: "81020bbf83364e462e9a7b4cbbbf414172cd8f8644bbd797a8ae399d99a7fb3e",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.9.0-windows-2025-x86_64.zip",
            sha256: "26e0c608960c8c745334c31c18de66aded33ef7208060e9e025d547e263585cc",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.9.0-windows-2022-x86_64.zip",
            sha256: "6f9c9b7b69216d8e1e04ad0c8ba2f715b82e98ac2522df6cd933a81a0fd16bc8",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.9.0-windows-11-arm64.zip",
            sha256: "ed9b44e0230dca91e2e6ebf8a739782a5497c4fb2691178fe626e624d58e8f69",
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
