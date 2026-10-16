//! Published TinySecurity admission pins copied verbatim from v0.12.0/checksum.toml.
//!
//! `LoadPolicy::Eager` is the "load at boot" policy (the issue calls it Boot): a missing
//! security module changes behaviour, so it must be serving before the first request.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSECURITY: ModuleRecord = ModuleRecord {
    id: "tinysecurity",
    description: "Native security policy and immutable filesystem authorization scopes",
    bus_name: tinysecurity_bus::names::INTERFACE,
    object_path: tinysecurity_bus::names::OBJECT_PATH,
    version: "0.13.0",
    release_url: "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.13.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysecurity-module-0.13.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "736b159ff71fe21eb8e780d18d974604e72d132482f3d97253e3c9f01de108c2",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysecurity-module-0.13.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "ab3bee8540ad9691d7237cc4730b5f528e69856773b3b0218e90de9c1ac98c95",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysecurity-module-0.13.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "46d06b759907110edeb6506c2d70e6d8d73d0c74103176937831605c8334a943",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysecurity-module-0.13.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "02a45efd8bff8ad6ca06ac3f4e1308d86325f8b2ec2076038807d58dd2c76920",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysecurity-module-0.13.0-macos-26-arm64.tar.gz",
            sha256: "e312b9f3f0561fea6a2171a82cdd9489561015d556d637118a13411311a4ab8d",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysecurity-module-0.13.0-macos-26-x86_64.tar.gz",
            sha256: "ebc046e138c7234f19d77d499d05f3c1632ed90751842f0517496009fcffd69b",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysecurity-module-0.13.0-macos-15-arm64.tar.gz",
            sha256: "9a9776846760525588d855333b997fe275227247e8761265cdd7732d31f1ac1e",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysecurity-module-0.13.0-macos-15-x86_64.tar.gz",
            sha256: "25c7b9e84b17d522987ae61e91736d0c9cfbd60ce70cdf03faf010333dbc2602",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysecurity-module-0.13.0-windows-2025-x86_64.zip",
            sha256: "3f8d877bb5c17a3fe48f2f8bd169831433839b151c65f1eb46e73c757c68d7d0",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysecurity-module-0.13.0-windows-2022-x86_64.zip",
            sha256: "17a24571c92ddb493a42a107b69d2423f59b521de28c486787615319700ed2f2",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysecurity-module-0.13.0-windows-11-arm64.zip",
            sha256: "81f5454e0493f2b5a9fb0ba4dcee028baa9ac355f1d5c2289e1ad5a1813748cc",
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
