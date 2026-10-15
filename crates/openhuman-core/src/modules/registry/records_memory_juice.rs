//! Registry record for the `tinyjuice` module.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYJUICE: ModuleRecord = ModuleRecord {
    id: "tinyjuice",
    description: "Content-aware tool-output compression and recoverable caching",
    bus_name: tinyjuice_bus::names::BUS_NAME,
    object_path: tinyjuice_bus::names::OBJECT_PATH,
    version: "0.10.4",
    release_url: "https://github.com/tinyhumansai/tinyjuice/releases/tag/v0.10.4",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinyjuice-module-0.10.4-archlinux-arm64.tar.gz",
            sha256: "428f9348146969cc640154cdb1378d1e632e598b78ca1ef4bae844133e32c25b",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinyjuice-module-0.10.4-archlinux-x86_64.tar.gz",
            sha256: "c61c4d38823cf547f757d36a57d0a1e7bfb175b243f74f81a91f78267fe9dac1",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinyjuice-module-0.10.4-fedora-43-arm64.tar.gz",
            sha256: "ab20b53696bdef7e8314abbc91b1f7d2238dd448ba55b1589a4b4412f70c4cca",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinyjuice-module-0.10.4-fedora-43-x86_64.tar.gz",
            sha256: "08142f4db18d339e2c5c53d418d2ef2ef79d674059cb44fbcc921530a844822e",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinyjuice-module-0.10.4-fedora-44-arm64.tar.gz",
            sha256: "1922eaa8c43c87827e560803cc3c2095e0d5f3cab8835e9d553962176c5cee11",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinyjuice-module-0.10.4-fedora-44-x86_64.tar.gz",
            sha256: "5cfe1964ffb5a4a235feec402863513b9aef77bdd5ed6a51d6c46187617decad",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyjuice-module-0.10.4-macos-15-arm64.tar.gz",
            sha256: "db2bf928f85125c23699f8d03768baa81261d85e18c77d5f50eba88d1f688e39",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyjuice-module-0.10.4-macos-15-x86_64.tar.gz",
            sha256: "8fa38225f5b9b85c74f504e4917e941d3d54c42e67930265d30a93408f74c882",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyjuice-module-0.10.4-macos-26-arm64.tar.gz",
            sha256: "52b176d6bf5f4b05f404046043eb75abcb44ff68a57f97c1bd23095324c9096c",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyjuice-module-0.10.4-macos-26-x86_64.tar.gz",
            sha256: "86cbe6f72ab59da93ce52ff87e5470cde99a1574413160635fcc8699144cdc07",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyjuice-module-0.10.4-ubuntu-22.04-arm64.tar.gz",
            sha256: "77a60cffad0701c4d8dab4f1c542313cc1c1fb4e194675e3b157529b237cf7bd",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyjuice-module-0.10.4-ubuntu-22.04-x86_64.tar.gz",
            sha256: "dfbbe83e687bc079c39d698df44af0305e2e4b5097e9655ac54465688d7bf3c9",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyjuice-module-0.10.4-ubuntu-24.04-arm64.tar.gz",
            sha256: "2d1e3e13ae91315bd5982868a8c2bf60930bf80454073796e2a6b79efcd2ee13",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyjuice-module-0.10.4-ubuntu-24.04-x86_64.tar.gz",
            sha256: "d54b830f603aab1c6b02a19a14a3ebd3c991bfa12ad6755b4811f6147554c40e",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyjuice-module-0.10.4-windows-11-arm64.zip",
            sha256: "9cf922b24e035c698f38941b641943b9112a253bde414c1620be7840262146fc",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyjuice-module-0.10.4-windows-2022-x86_64.zip",
            sha256: "b42bc63bfed347d6e374d5f3556cd92fd1e9180d0fa325b1e613681ba94605a1",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyjuice-module-0.10.4-windows-2025-x86_64.zip",
            sha256: "f991c0d8ff03531dc590c55d1666f0e7b24993a1b070fdeb7549d20611502f89",
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
