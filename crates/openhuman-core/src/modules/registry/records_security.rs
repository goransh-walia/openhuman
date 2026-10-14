//! Published TinySecurity admission pins copied verbatim from v0.4.1/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.4.1",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.4.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.4.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "5cf70265a35c3fcf6fe138e41d9013aa5ea4c137ceddc8c365fcda9567127200",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.4.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "c29e4fc6c6e8be978d693d005da8af447be73e623ccee00349a9af8c9f03afdb",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.4.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "d45a08c6805e9a65a063065799edaad82a0031ced5c8cdd49b1fa8649376476c",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.4.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "0def3204fa002baa9d54875776c8ba782c6662593e15619050350d2e1c2895bc",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.4.1-macos-26-arm64.tar.gz",
            sha256: "27654bfc3ef3fbbaec52b2ddff164a529222046e0b47acbb2cdaa0e4aea36859",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.4.1-macos-26-x86_64.tar.gz",
            sha256: "2a235020459b1cbf391c61a39cbb174f77f5075b1aa005c26897e322bd575328",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.4.1-macos-15-arm64.tar.gz",
            sha256: "8cd3e998da562564b01f452f4a4feed091564e770f2504144b9463378536eb47",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.4.1-macos-15-x86_64.tar.gz",
            sha256: "ce9d3e9168d138ed84bedb81b3405c23207280981b3944252b7149bf473effc2",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.4.1-windows-2025-x86_64.zip",
            sha256: "86a0ff9dd8181413310e7d7c765d214140c234ecde042c9afd887db9bed3a207",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.4.1-windows-2022-x86_64.zip",
            sha256: "172add2931c3c5a605696cdea609edbae180c035f19d086172ab1acacfe2b829",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.4.1-windows-11-arm64.zip",
            sha256: "db2b00a6d87db0cebd7f5a5ff201ed33237014c07956edf6034f0a506ddc432a",
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
