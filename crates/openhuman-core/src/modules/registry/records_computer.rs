//! Native computer-use module (desktop and browser control). Published digests are added only from its release manifest.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};
use tinycomputer_bus::names;

pub(crate) const TINYCOMPUTER: ModuleRecord = ModuleRecord {
    id: "tinycomputer",
    description: "Permission-aware desktop and browser observation and control",
    bus_name: names::INTERFACE,
    object_path: names::OBJECT_PATH,
    version: "0.13.0",
    release_url: "https://github.com/tinyhumansai/tinycomputer/releases/tag/v0.13.0",
    // All seventeen published archives were checked against checksum.toml
    // and GitHub asset digests before pinning. Linux permission probes are
    // part of the shipped voice surface alongside macOS and Windows.
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinycomputer-0.13.0-archlinux-arm64.tar.gz",
            sha256: "fadaa7be03eb1ee9032bee3c6de67139e4b08499c5d8c951dd994ff5acc85606",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinycomputer-0.13.0-archlinux-x86_64.tar.gz",
            sha256: "dda869d8e947327bdd724a9265fbab6d69ddeed43f000b2dded10858e30c117d",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinycomputer-0.13.0-fedora-43-arm64.tar.gz",
            sha256: "0169e7f2cac06cf38d07e37173fe29145bfb5b5bfa37fc3efe115ec7b6b39cc2",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinycomputer-0.13.0-fedora-43-x86_64.tar.gz",
            sha256: "bb01196c6e8b00e25a6facc3106efc82a335c41c59cf91545afefa534a9282fa",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinycomputer-0.13.0-fedora-44-arm64.tar.gz",
            sha256: "0592e2d9ccd4ed68cbfa7879276dfd75c0c734f84915c0ee47181955751277c7",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinycomputer-0.13.0-fedora-44-x86_64.tar.gz",
            sha256: "025944aee011f9ce264a01ae6bff76eb03ece92d15e6d9b8cea75253addf8a34",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinycomputer-0.13.0-macos-15-arm64.tar.gz",
            sha256: "14e920c54478b2bc66a04665293932f8072076c17d12cf0613d6cb023f24064d",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinycomputer-0.13.0-macos-15-x86_64.tar.gz",
            sha256: "f2d94a229577e545cf471c187830ec63342a3d4d7dfa61de04aa08748b88a802",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinycomputer-0.13.0-macos-26-arm64.tar.gz",
            sha256: "c8a02a2f7e87ccabfea5f4c0d06c03e1785d3a5b7dc5885ec23ba459d23182da",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinycomputer-0.13.0-macos-26-x86_64.tar.gz",
            sha256: "da00b8301024b3518d1fc1b7e5f8567bcb261e95b407e66117ca9c5e7b802ae6",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinycomputer-0.13.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "13be2989a25fcdebd952e473d5db116277a43bfc16fd5a84d7e1169a98f60386",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinycomputer-0.13.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "c58f223a48c6da093a6f449898d738562f7b01667e4dff239437271f2972e348",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinycomputer-0.13.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "6944a2002fb715f99c9b8294d047efb3b9d166dbc67b64b46f83a4fe459f027d",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinycomputer-0.13.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "b1e1d60c2e9d79153cb77831824e04b2b4d80297d0a4428ec99bcb0c5ca4c80e",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinycomputer-0.13.0-windows-11-arm64.zip",
            sha256: "47d921484a28264b929a76fc3a41c9afd9978fb138c2be20501d09b798314fdb",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinycomputer-0.13.0-windows-2022-x86_64.zip",
            sha256: "8f8da3217b5b28899859a156240947031b8787865ad31ed70cc3152347121f0d",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinycomputer-0.13.0-windows-2025-x86_64.zip",
            sha256: "e1a7c72549b0f3e5d3b51c41f14d3ef625df3d2a0c9db143602e23e196af9fa9",
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
