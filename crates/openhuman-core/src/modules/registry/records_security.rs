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
    version: "0.8.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.8.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.8.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "a62ef9ed6b98a298378f80e89f66c618e3144240d18549ac635a27ff84c66e3e",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.8.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "77285180885c5caa7fda5dbda62f67b690a3ada51fefa8fc866c2aa50bcde26a",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.8.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "3092fcc9c829498278ca30554d41a2b351f6fcf8c1ee92766c3832a255a8af32",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.8.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "d17da91b22f4e6f35495e4e6ea9093dcbc88f068e2251ec54214f763fc381cf0",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.8.0-macos-26-arm64.tar.gz",
            sha256: "77a48336d5aaba44750cf4190b6857d65b57e3c58c91daf42842729ebda3910d",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.8.0-macos-26-x86_64.tar.gz",
            sha256: "bb90011b0f06ecbc272be29ff2f204be256567b6d6c4734374f5e4c82389fd1a",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.8.0-macos-15-arm64.tar.gz",
            sha256: "84cac05a13362f805eb35e1d21d13d8c05a6cf6da444a7e2d74e9408df0f08a9",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.8.0-macos-15-x86_64.tar.gz",
            sha256: "a4a06dcf642e492e3c98f24c17682488f8218c3e5a992b2f1a7d19dfa0488033",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.8.0-windows-2025-x86_64.zip",
            sha256: "7ad1fd1fa7429bd6339942fb84c3af3760b20859309db82652a9f4bb1b123cda",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.8.0-windows-2022-x86_64.zip",
            sha256: "40f85e906ed18d40ca8b7816bd19fc547755452f1053b8fedfb3c94c6742b552",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.8.0-windows-11-arm64.zip",
            sha256: "e6e4f4b395ce43764e0988765a08782f89054fa7e522d4fa86259c743edf2abd",
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
