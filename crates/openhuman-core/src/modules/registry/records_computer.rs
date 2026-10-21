//! Native computer-use module (desktop and browser control). Published digests are added only from its release manifest.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};
use tinycomputer_bus::names;

pub(crate) const TINYCOMPUTER: ModuleRecord = ModuleRecord {
    id: "tinycomputer",
    description: "Permission-aware desktop and browser observation and control",
    bus_name: names::INTERFACE,
    object_path: names::OBJECT_PATH,
    version: "0.15.0",
    release_url: "https://github.com/tinyhumansai/tinycomputer/releases/tag/v0.15.0",
    // All seventeen published archives were checked against checksum.toml
    // and GitHub asset digests before pinning. Linux permission probes are
    // part of the shipped voice surface alongside macOS and Windows.
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinycomputer-0.15.0-archlinux-arm64.tar.gz",
            sha256: "9594283a45e3f99179e4a578fee7f9f534c79079d64a66f7509dd2af2ba29540",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinycomputer-0.15.0-archlinux-x86_64.tar.gz",
            sha256: "47f7ef561b52ea268209f760217812ae289a25c9aa9b32494b1626a90f7c8d7e",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinycomputer-0.15.0-fedora-43-arm64.tar.gz",
            sha256: "14ee1251f8c08c6d40f36154bc051f7d4437eceb9bbe859fab20eca19a8a463e",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinycomputer-0.15.0-fedora-43-x86_64.tar.gz",
            sha256: "1aba06a3c89c010cf2a185e031ece42303286f705f78b57d6950bc69bfe24337",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinycomputer-0.15.0-fedora-44-arm64.tar.gz",
            sha256: "5909114d9ad1298dea846e58fafe87c2d9e2b33f188843858aa6df4bf4d9845b",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinycomputer-0.15.0-fedora-44-x86_64.tar.gz",
            sha256: "afed59c358d01e1aff00880c623c8c0aa351b805783f93cb129cfb7912b1718c",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinycomputer-0.15.0-macos-15-arm64.tar.gz",
            sha256: "c09efc9942a9242d47ea72513f1cc547fe5ff98e5cb1f20615307481965662d5",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinycomputer-0.15.0-macos-15-x86_64.tar.gz",
            sha256: "e0ec88e2d4767388c348dc098980dc4430b78efaac5622fc2c3b106853583119",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinycomputer-0.15.0-macos-26-arm64.tar.gz",
            sha256: "9bacf2451dc9b1113bd5293f268743048bbe4e92db4f343b9e6a9dadea68249b",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinycomputer-0.15.0-macos-26-x86_64.tar.gz",
            sha256: "20d4b2db0b3c3c0f816996074ce766618e563bd3f11e5b98c0a7213b28a23c47",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinycomputer-0.15.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "8941f58d64fdb5c7e11d5d1ca32ce8db280ee6e976fbcc5b9781a31906565aad",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinycomputer-0.15.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "02b4bda6743d83dad400fdc57ed63eecf7c5efe113861a163ad9389d0a619749",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinycomputer-0.15.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "9d25a2456010cf624480449f70b5763e38cdc8eccbe7a7a1aa6e7e04b9528123",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinycomputer-0.15.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "c33256574bbffcff991d1da852c10f7ae5b5b838fe94ce70d801bdaa0975e190",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinycomputer-0.15.0-windows-11-arm64.zip",
            sha256: "8791e25d19387aa1ccc8e3252aba2aaae3182e847e36b6da98bf2ab651e2a952",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinycomputer-0.15.0-windows-2022-x86_64.zip",
            sha256: "2fe46f2e6d4c1759676e807b74c6353e63dae3d1c4b71e5e7d32c1ddb5cf783c",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinycomputer-0.15.0-windows-2025-x86_64.zip",
            sha256: "930967e93bb9bd00f369bcd1abeaa14d938f252101e6ca69fd5a6127f87a22e4",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinycomputer",
        manifest: "crates/tinycomputer/Cargo.toml",
        package: "tinycomputer",
        lib_name: "tinycomputer",
        features: &[],
    },
};
