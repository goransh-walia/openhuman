//! Registry record for the `tinyjuice` module.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYJUICE: ModuleRecord = ModuleRecord {
    id: "tinyjuice",
    description: "Content-aware tool-output compression and recoverable caching",
    bus_name: tinyjuice_bus::names::BUS_NAME,
    object_path: tinyjuice_bus::names::OBJECT_PATH,
    version: "0.10.3",
    release_url: "https://github.com/tinyhumansai/tinyjuice/releases/tag/v0.10.3",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinyjuice-module-0.10.3-archlinux-arm64.tar.gz",
            sha256: "cde7f906045aa247d253cca9d1a4d4bde7715fb7f4318f48f8b0313805c9132c",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinyjuice-module-0.10.3-archlinux-x86_64.tar.gz",
            sha256: "3eaaf8db6d3a3b15ff2c25c6078a0963064f68d3cab98f6ebea2ed12ea9dba81",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinyjuice-module-0.10.3-fedora-43-arm64.tar.gz",
            sha256: "e9f14fd2df5536e720f5de245bd1936ffa22337c3ac182b3693c49aae022b975",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinyjuice-module-0.10.3-fedora-43-x86_64.tar.gz",
            sha256: "deafac60c2ca5288e7f801c3e79d73623a5275af244d701b817d01f20736e1f6",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinyjuice-module-0.10.3-fedora-44-arm64.tar.gz",
            sha256: "3a37d715e4f317e8ea94b8158e406492c720f712d5266e5d9692908e24146267",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinyjuice-module-0.10.3-fedora-44-x86_64.tar.gz",
            sha256: "97bf363524e80a24fefe8ad2133e6adec27bbf9085c7342598d2e64657441e46",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyjuice-module-0.10.3-macos-15-arm64.tar.gz",
            sha256: "016950d1c1d184386f1531846d4c514e246d1e291707b396acb72e0205e507e4",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyjuice-module-0.10.3-macos-15-x86_64.tar.gz",
            sha256: "8dd4b5f671a2ec7933923e42d441fc67a7254dea21160c53bae41eab97eb7081",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyjuice-module-0.10.3-macos-26-arm64.tar.gz",
            sha256: "864b5d52db0f9141b7226ef694aed4616fe65d9c4f6c7093908cea35d5635072",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyjuice-module-0.10.3-macos-26-x86_64.tar.gz",
            sha256: "8ae54221371bd8971a3b3dc4e80da4be6ba8da3afac34a1926db00a1a2cf95f9",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyjuice-module-0.10.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "ffd216e948280f1c8d9b84518c651c6075fac39b94b1198236c0115df804c382",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyjuice-module-0.10.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "73a7b0279f511b57e9d8188816bc5bf5b66c307d14fe933fb6f3cf98b05e59a6",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyjuice-module-0.10.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "a980c9cf72df6b92fbd8cb40b53a06ad6f60dfe09af31afc75b2648983b5aef6",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyjuice-module-0.10.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "b4175c95a1d44bad5b6fc488c02202e8485288413c6e01bdc6d6956197797aaa",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyjuice-module-0.10.3-windows-11-arm64.zip",
            sha256: "a12aefcaefd66402bf3d9f26ae816b349b6977d9576e9934f1c98cd345442fbc",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyjuice-module-0.10.3-windows-2022-x86_64.zip",
            sha256: "e1c66c93c6c19e36bc0207e37b90d1b09395b2f4e07a8b782af72c8f8d8256f3",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyjuice-module-0.10.3-windows-2025-x86_64.zip",
            sha256: "e8d97c082f56337bf639813ea3507bd5c9874b2c2e6e78a32dc39a405b431117",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinyjuice",
        manifest: "crates/tinyjuice-module/Cargo.toml",
        package: "tinyjuice-module",
        lib_name: "tinyjuice_module",
        features: &[],
    },
};
