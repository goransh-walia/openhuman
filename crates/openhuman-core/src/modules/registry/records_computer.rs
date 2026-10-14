//! Native computer-use module (desktop and browser control). Published digests are added only from its release manifest.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};
use tinycomputer_bus::names;

pub(crate) const TINYCOMPUTER: ModuleRecord = ModuleRecord {
    id: "tinycomputer",
    description: "Permission-aware desktop and browser observation and control",
    bus_name: names::INTERFACE,
    object_path: names::OBJECT_PATH,
    version: "0.13.1",
    release_url: "https://github.com/tinyhumansai/tinycomputer/releases/tag/v0.13.1",
    // All seventeen published archives were checked against checksum.toml
    // and GitHub asset digests before pinning. Linux permission probes are
    // part of the shipped voice surface alongside macOS and Windows.
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinycomputer-0.13.1-archlinux-arm64.tar.gz",
            sha256: "74ec195d2c787643262baa3a2dda629dd20169a2372c2a855a737fe3c34c51f9",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinycomputer-0.13.1-archlinux-x86_64.tar.gz",
            sha256: "840391f5319e57840527c35daf56d24435dc02ac35813846ba98c503c3195284",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinycomputer-0.13.1-fedora-43-arm64.tar.gz",
            sha256: "f92be46a55b1937d79412c0049c6b53ff1ba0468ef6eb512be7b88a91201464d",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinycomputer-0.13.1-fedora-43-x86_64.tar.gz",
            sha256: "3cf4c169ba3a9fc033e0b10daae9998db0b17d6bb36f3771ea81907cfbd6ad3e",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinycomputer-0.13.1-fedora-44-arm64.tar.gz",
            sha256: "0123138e73d29ddca44441719dbfefc18b073a39fa27d6edbad13937defd2709",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinycomputer-0.13.1-fedora-44-x86_64.tar.gz",
            sha256: "c41d8657263ff6a26754fd9664ccf4e0b14455ab7b685ebca3d3f29f33f615c8",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinycomputer-0.13.1-macos-15-arm64.tar.gz",
            sha256: "49fa68fe1f4ebd0055b4177f606f2080116a0637e7cb4ccd59de43f020876ec5",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinycomputer-0.13.1-macos-15-x86_64.tar.gz",
            sha256: "93c079bb780a0933e671edd806ea74a1b3775508773a2e24f1cfcd31f9cb2246",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinycomputer-0.13.1-macos-26-arm64.tar.gz",
            sha256: "6b4965e63e3c8f666f9aa9abf12fb11d172c23097e95aee08688629dc6fdacd5",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinycomputer-0.13.1-macos-26-x86_64.tar.gz",
            sha256: "94ca270b74237a56d1af6a4ccca3fe163bf1c547ae27d75eab6a06b39c4ffc24",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinycomputer-0.13.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "b4e51c7da5c9fb5467afe9ac94fb6727f073d623201356faf3e74d5638c2c165",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinycomputer-0.13.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "0df84458e7d86bc1281571bac80c61e9847e07b059abaefa3dad3eb436454f8d",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinycomputer-0.13.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "bc8f4369c52fd59584de9d7f0d5fe825c454bb96b93cbf6c77d15c8c4d2da1bd",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinycomputer-0.13.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "bf96e87011844af29aa065f6ad95fd0f9bfd1d3a904dca4843ab41bb59b9c873",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinycomputer-0.13.1-windows-11-arm64.zip",
            sha256: "3e79670da59a24a1ff24d0adfb3506ce5ec506922db1024ef1498cd56a94237c",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinycomputer-0.13.1-windows-2022-x86_64.zip",
            sha256: "fcaf3f3852fff807a97f8b00abeac9a0537a827709ab78f96cbacad80fed8793",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinycomputer-0.13.1-windows-2025-x86_64.zip",
            sha256: "5f27e27178a78e61eb7ed216e561bfe040e86d94846fd33651c43312b63ec843",
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
