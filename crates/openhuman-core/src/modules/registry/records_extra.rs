//! Registry records for additional first-party TinyBus modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

/// The `tinybox` module, loaded on demand.
#[cfg(feature = "modules")]
pub(crate) const TINYBOX: ModuleRecord = ModuleRecord {
    id: "tinybox",
    description: "Sandbox execution and lifecycle operations through TinyBox",
    bus_name: tinybox_bus::INTERFACE,
    object_path: tinybox_bus::OBJECT_PATH,
    version: "0.1.36",
    release_url: "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.36",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinybox-0.1.36-archlinux-arm64.tar.gz",
            sha256: "e9dc897eaf1def46494420d9e68a2924cffffb7d4d769279f9ad03d563d1e8b2",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinybox-0.1.36-archlinux-x86_64.tar.gz",
            sha256: "6d5bef1b7f640749d7f472ea91a9172df3b9f764e9688ba0a0480e922c7ee6a4",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinybox-0.1.36-fedora-43-arm64.tar.gz",
            sha256: "5bb6672f729b850a8aefd14b2feb28daa2b51819fcc13f7878d23f5857802a44",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinybox-0.1.36-fedora-43-x86_64.tar.gz",
            sha256: "99f187bb06d71c7f2e811885a240be7f97256a32201553db4e2cdf593798a29d",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinybox-0.1.36-fedora-44-arm64.tar.gz",
            sha256: "a762cbd652f893e62abe5645ca1f6df82ac3f2cd8215ed8fe3e289720deff496",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinybox-0.1.36-fedora-44-x86_64.tar.gz",
            sha256: "06520c311675378c009288723807e031040fc91964f47268d435d007d2a3aa5a",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybox-0.1.36-macos-15-arm64.tar.gz",
            sha256: "67a30c43ddef3ff68981daaad8a7af8a68b81f51fc9e44b77fdef2747c164018",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybox-0.1.36-macos-15-x86_64.tar.gz",
            sha256: "e0444a4224551823c7c558831488969c5c3921e07a1d944fc484d08ffe928140",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybox-0.1.36-macos-26-arm64.tar.gz",
            sha256: "449cf189d430427e9ed85b9d8160bae5d6e783d978b94c31178c11bcb95d15e6",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybox-0.1.36-macos-26-x86_64.tar.gz",
            sha256: "b5d8807c3443618767f0ace38e9fd9a1b03cefa0a5db1f697e26b229967d0f0f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybox-0.1.36-ubuntu-22.04-arm64.tar.gz",
            sha256: "1045bad40d5703349df89bf8d52d8802d3e782818609d2d086720974c0631f11",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybox-0.1.36-ubuntu-22.04-x86_64.tar.gz",
            sha256: "d324e1308b8e30fbd7050eecbe8b53ec73d323789d6d4d2cd9af8a503e3e0e66",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybox-0.1.36-ubuntu-24.04-arm64.tar.gz",
            sha256: "2f9f2931168580e16ea35d1fe32f00fc019f4d67caaf07fbf117c06feeae69b6",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybox-0.1.36-ubuntu-24.04-x86_64.tar.gz",
            sha256: "1fc7a35aad64a5dfdeb6030e5c650338fd49ccaa5e068311d49b8f6d503234a9",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybox-0.1.36-windows-11-arm64.zip",
            sha256: "e230fb5802a766bb55b662e3e5ed287b6e57548e80c376bc5ca6f786e2b5b24a",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybox-0.1.36-windows-2022-x86_64.zip",
            sha256: "b6ebc5f77536d42c51f58b4d35e671f185de1a9dbe0ac6ed8978a4e66573a261",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybox-0.1.36-windows-2025-x86_64.zip",
            sha256: "f6a23c2e4eefc12dfc359a5d6993b7335730e617fe1304beec47081a067ed804",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinybox",
        manifest: "crates/tinybox-module/Cargo.toml",
        package: "tinybox-module",
        lib_name: "tinybox",
        features: &[],
    },
};

/// The `tinychannels` module, loaded on demand.
pub(crate) const TINYCHANNELS: ModuleRecord = ModuleRecord {
    id: "tinychannels",
    description: "Channel provider lifecycle and message transport",
    bus_name: tinychannels_bus::names::BUS_NAME,
    object_path: tinychannels_bus::names::OBJECT_PATH,
    version: "0.1.34",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.34",
    assets: &[
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.34-macos-15-arm64.tar.gz",
            sha256: "b58f68ac45af467fa9eea0443afa59ed17a2f4e7ab9c3f4223ded6d83af88b22",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.34-macos-15-x86_64.tar.gz",
            sha256: "ec04b49b184388b514ffc4aad9e130141987bd28d9cc5f4a76daf87c25e28a98",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.34-macos-26-arm64.tar.gz",
            sha256: "7b4f3be24e83a056e68d072cc1a5e00420d18f30eef0cd767cb6be40d0b2ad95",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.34-macos-26-x86_64.tar.gz",
            sha256: "08e3d4ad69b30c46b06f4846133bb8066396e95f8f161f98c7d20d375b50453c",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.34-ubuntu-22.04-arm64.tar.gz",
            sha256: "035d782536952a7e60d3ba9011ee96cca71e63d515ef78e7397d4800ecf3aece",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.34-ubuntu-22.04-x86_64.tar.gz",
            sha256: "98292be5f921c027a79e41b463ffc9981fca139e9c269bb8daf882392c97817c",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.34-ubuntu-24.04-arm64.tar.gz",
            sha256: "57af460080ecdd14970e47dc02d8f27334cf165ab70ba3fb82e34dc88e445329",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.34-ubuntu-24.04-x86_64.tar.gz",
            sha256: "e153a65569f3e7aba90f78c3fe6ada4684280ea02776d5263a9df19071335fc7",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.34-windows-11-arm64.zip",
            sha256: "ef163bcd82fcdeafbb364f1f287eda0e9778255e04fb4062534ccf365f87bdeb",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.34-windows-2022-x86_64.zip",
            sha256: "a570821ef804e71ad98c747e51a22b8631a7d3db760e99b240cf666fafe67ced",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.34-windows-2025-x86_64.zip",
            sha256: "95bbf5aca663af06a6e6b912fa519e249fd1f454c65c0bb2993d1e1a79973316",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinychannels",
        manifest: "crates/tinychannels-module/Cargo.toml",
        package: "tinychannels-module",
        lib_name: "tinychannels_module",
        features: &[],
    },
};

#[cfg(feature = "hosting")]
/// The `tinyhosts` module, loaded on demand.
pub(crate) const TINYHOSTS: ModuleRecord = ModuleRecord {
    id: "tinyhosts",
    description: "Hosting provider operations",
    bus_name: tinyhosts_bus::BUS_NAME,
    object_path: tinyhosts_bus::OBJECT_PATH,
    version: "0.4.1",
    release_url: "https://github.com/tinyhumansai/tinyhosts/releases/tag/v0.4.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyhosts-0.4.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "7e8a72b7951db71cc98c94fed0908660a58bd3f2cbf23103d953e28e1315d4d5",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyhosts-0.4.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "27cbc5a233292850797adc7b51d9d654a10d4aa77ec285a7e564e378889e0a03",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyhosts-0.4.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "a03588ef8b012aea29f2b2b7105fde6c3c7b4bb0dc6f93d99868e91468c2b6d6",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyhosts-0.4.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "80b26591eb7beff4e27de9e6239b919ff6c17b00ead11d22bae8e7fcb0d33c15",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyhosts-0.4.1-macos-26-arm64.tar.gz",
            sha256: "ba40e889ebd450edd791865945b4146b2f243bc885f4228463e65b0a4ea8074e",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyhosts-0.4.1-macos-26-x86_64.tar.gz",
            sha256: "eeb2a66603374934724036e9fb86fcba0c22a2fd3aa168127cca5960eb5a19aa",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyhosts-0.4.1-macos-15-arm64.tar.gz",
            sha256: "b67a0aecb0527af68d5b5428d55d897ece11500e2ee9798b3d133ef0df7344b7",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyhosts-0.4.1-macos-15-x86_64.tar.gz",
            sha256: "7582dc327e247b28e657a01c8f8a4cb0924c8211144dfb24e4917868cd237354",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyhosts-0.4.1-windows-2025-x86_64.zip",
            sha256: "42c2a5b98eb5c51a8a05731e762af801c585383024d27cf4cd8bb01f9014308f",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyhosts-0.4.1-windows-2022-x86_64.zip",
            sha256: "0d579e0d08856307881b8965bf78762f0064b129861e13e796f91fcb9895ccaf",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyhosts-0.4.1-windows-11-arm64.zip",
            sha256: "67b64f0917fa0cf561aa08dfe0c74818e9ac9066a9f03c03e6b216f7af87aea3",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinyhosts",
        manifest: "Cargo.toml",
        package: "tinyhosts",
        lib_name: "tinyhosts",
        features: &[],
    },
};
