//! Registry records for additional first-party TinyBus modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

/// The `tinybox` module, loaded on demand.
#[cfg(feature = "modules")]
pub(crate) const TINYBOX: ModuleRecord = ModuleRecord {
    id: "tinybox",
    description: "Sandbox execution and lifecycle operations through TinyBox",
    bus_name: tinybox_bus::INTERFACE,
    object_path: tinybox_bus::OBJECT_PATH,
    version: "0.1.32",
    release_url: "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.32",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinybox-0.1.32-archlinux-arm64.tar.gz",
            sha256: "c772e031b51dfa022acb056ae5be30305b10daafa8548bb8171fb70df0fdd4aa",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinybox-0.1.32-archlinux-x86_64.tar.gz",
            sha256: "c710c996b8ae3b0c34c1f399fef3c3276d5d0fe2c5c2a02ea9b1815f277dd18c",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinybox-0.1.32-fedora-43-arm64.tar.gz",
            sha256: "bad9ef8ee84b5c22a896742ac25bd51a006690d698f8160ea96e77fde8e98d74",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinybox-0.1.32-fedora-43-x86_64.tar.gz",
            sha256: "7538c68dd00a24ce9dae71996afdb979198475838af14d03da13b38b043ab922",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinybox-0.1.32-fedora-44-arm64.tar.gz",
            sha256: "092491e457a8be57a650b21198282cbce79630b9fdd082eff0bb8490e2412742",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinybox-0.1.32-fedora-44-x86_64.tar.gz",
            sha256: "c2565a557a806f7782a23e199007494df245087f7b16689c59cff3efb67df384",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybox-0.1.32-macos-15-arm64.tar.gz",
            sha256: "db20c950d43e77d2255110cc289594371a957ffb073dea24de10e4ec68949158",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybox-0.1.32-macos-15-x86_64.tar.gz",
            sha256: "93b5f3296fe2b41c40343cf845289705cf02081a5fca26fafea5c0ebb161c843",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybox-0.1.32-macos-26-arm64.tar.gz",
            sha256: "b387f65e0206821edfa0b4ff1459a407261c97d2fa7ee332e5ce9a48e9f667e1",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybox-0.1.32-macos-26-x86_64.tar.gz",
            sha256: "1258a4ee08335a402989c265a66421fa466a43fd19798fcd13a9672a44acb338",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybox-0.1.32-ubuntu-22.04-arm64.tar.gz",
            sha256: "8fcec28c0c6c168b2ed2085e786d5eb63af783d2be4284d76e6366f5a0055b14",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybox-0.1.32-ubuntu-22.04-x86_64.tar.gz",
            sha256: "641d5463d642bef40d25978d8e19f5bd0a2492e664a12d9d60b0ec9b2c8cf49a",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybox-0.1.32-ubuntu-24.04-arm64.tar.gz",
            sha256: "52541665159e1a06b1102ee9fed198d8cc9822fa045b3e813e8e9b98a048b9cb",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybox-0.1.32-ubuntu-24.04-x86_64.tar.gz",
            sha256: "a537795061172a57c9269a530ba84ee39e184f01aa433d2254b20b0c6408ec25",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybox-0.1.32-windows-11-arm64.zip",
            sha256: "356b737eefaa6132bcb226692aa9681b6ead6b5841d83f0130be034a25bed225",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybox-0.1.32-windows-2022-x86_64.zip",
            sha256: "172a69e61861a98be91245a82179292f564fb682d180af219f2dddbce54354c6",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybox-0.1.32-windows-2025-x86_64.zip",
            sha256: "24c8fe29cb4659d79a1cb4a1ed9635fa0db8273f0e7dfe0880ee764032e4dc85",
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
    version: "0.1.33",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.33",
    assets: &[
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.33-macos-15-arm64.tar.gz",
            sha256: "770cd74e7d6586aa484383d66d7b51fcd29cc5ab417c11bee0af69b0c6ab45ee",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.33-macos-15-x86_64.tar.gz",
            sha256: "12e3a51b0e4bef0cd53b290daa950cbe1f3d01f7f5f38b69a72a0addda3d91e8",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.33-macos-26-arm64.tar.gz",
            sha256: "5a88af85efd608b13c7a5cb79153c66a417fb69bc35712a72c49bbb029ecf473",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.33-macos-26-x86_64.tar.gz",
            sha256: "c2d632d72e0927d241fb14b35a587c4b9ce1ff4bd35fd3fb5c06817059ef7578",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.33-ubuntu-22.04-arm64.tar.gz",
            sha256: "742b1aebbe5e3d383302aeb7a05f4fcbb6c8822fb882adb7431140a720dd94b3",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.33-ubuntu-22.04-x86_64.tar.gz",
            sha256: "ebf0706cfcfd13f1099f225de279d9a28d290db3c3f1c93054a9048509759cc5",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.33-ubuntu-24.04-arm64.tar.gz",
            sha256: "4ff242d961bb7a43e0f077def8d19fa43426f4e101772a966a1ff9f448408256",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.33-ubuntu-24.04-x86_64.tar.gz",
            sha256: "e9e071303dab85fc5421c222ce1aa7e16f0af9301f3c7547fee0cb266b2f1da5",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.33-windows-11-arm64.zip",
            sha256: "19ba7ee1b91c5c519b3a65809d3a0459b7df572f3e1916c647e65292eebd18c4",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.33-windows-2022-x86_64.zip",
            sha256: "7a9499773c190d137ad31572947b7843b591efd45276675efeaa76c57d158f24",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.33-windows-2025-x86_64.zip",
            sha256: "fc1954c4258175bd3ff27780a18ad6fa064176ee84e19f076bd327e86ff87f3b",
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
