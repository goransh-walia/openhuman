//! Registry records for additional first-party TinyBus modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The `tinybox` module, loaded on demand.
pub(crate) const TINYBOX: ModuleRecord = ModuleRecord {
    id: "tinybox",
    description: "Sandbox execution and lifecycle operations through TinyBox",
    bus_name: "ai.tinyhumans.tinybox.Box",
    object_path: "/ai/tinyhumans/tinybox/Box",
    version: "0.1.24",
    release_url: "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.24",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinybox-0.1.24-archlinux-arm64.tar.gz",
            sha256: "11b4c959868e9e13f60af116946145530ca76ad68ab2ceef4fea11a0c2f76c5f",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinybox-0.1.24-archlinux-x86_64.tar.gz",
            sha256: "84764f24331dfb5ff1798419e69ada288233506f6d28f4f582f5e6a87f65d9e9",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinybox-0.1.24-fedora-43-arm64.tar.gz",
            sha256: "26763cb6a77e638adfb8641b58aaa0f57493403c7ab4dbca639f35e434fbd20c",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinybox-0.1.24-fedora-43-x86_64.tar.gz",
            sha256: "442d67e92c011125350e672a505925008b0bb26c368d3408c24029e60bc12be9",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinybox-0.1.24-fedora-44-arm64.tar.gz",
            sha256: "67c0bf40748a1bf9e749bc918cdaddc5ba53f29f9cc3327a7c56fc82ef8dfdee",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinybox-0.1.24-fedora-44-x86_64.tar.gz",
            sha256: "5e320dcce68a3b36c61a33647575c9e3f1449388f489e6aa094b7684863ab377",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybox-0.1.24-macos-15-arm64.tar.gz",
            sha256: "59a76b8ea5ce093d26616b115cd11493459dcefebec73c2b63fa115e4c70d6f7",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybox-0.1.24-macos-15-x86_64.tar.gz",
            sha256: "dd73c660e3c54004dad55257eeefc812fa8c2b3bf89d82e346b5bb4e95439b7f",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybox-0.1.24-macos-26-arm64.tar.gz",
            sha256: "3c2091b15246cb6fb566c73cbfc2f1a334b9f1d1a4ea58839f3576536321b285",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybox-0.1.24-macos-26-x86_64.tar.gz",
            sha256: "a077fae96405163fe4cc2ae0dcdfb2dee9e2b8da8ea7f7122d180d0c2f3789eb",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybox-0.1.24-ubuntu-22.04-arm64.tar.gz",
            sha256: "beb26115d378dce167ffe6970f57dfd268c9851f10e254306812be3f478f4567",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybox-0.1.24-ubuntu-22.04-x86_64.tar.gz",
            sha256: "be69717def1d836a8e64e3ed544fdb4fb772ef641df68b473e9677f030f94809",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybox-0.1.24-ubuntu-24.04-arm64.tar.gz",
            sha256: "4935e8099f1a373b7d3fffc6f5a4019d3e40a4c5c3d360e30c1d311ab6eba777",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybox-0.1.24-ubuntu-24.04-x86_64.tar.gz",
            sha256: "33391c0e42a5dc48328bb3cb2f785c44f4fc056ec5fc125a655c1416669b233e",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybox-0.1.24-windows-11-arm64.zip",
            sha256: "9b00a82bacbba00f528aaf6655bc005fa96751f774cb41cf960cb5c1ac0face9",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybox-0.1.24-windows-2022-x86_64.zip",
            sha256: "a5b676f18bc1252ee81d614c35fd2b1ffb2b2ecc3a0335face170d2185432255",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybox-0.1.24-windows-2025-x86_64.zip",
            sha256: "c09e9d417442be2748cea8f6a1fe17b323675400f3e8b82dc647ec5a84eb3b99",
        },
    ],
    load: LoadPolicy::Lazy,
};

/// The `tinychannels` module, loaded on demand.
pub(crate) const TINYCHANNELS: ModuleRecord = ModuleRecord {
    id: "tinychannels",
    description: "Channel provider lifecycle and message transport",
    bus_name: "ai.tinyhumans.tinychannels.Channels",
    object_path: "/ai/tinyhumans/tinychannels/Channels",
    version: "0.1.26",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.26",
    assets: &[
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.26-macos-15-arm64.tar.gz",
            sha256: "5924dc78a1b2931c526b5ca2590c675455eaca6db914c648a321ea9225d65142",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.26-macos-15-x86_64.tar.gz",
            sha256: "9be7ad6b11e9dcf14cbe8dba246e8aab55da29b03b60c36cd1a81dc250e312ac",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.26-macos-26-arm64.tar.gz",
            sha256: "466d6ceaf91735d278e65280f315efa5cfec7e609c9fe12cb8ff4e545846a4e3",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.26-macos-26-x86_64.tar.gz",
            sha256: "2f3636ac9bf0b5fadd4b2539730064437770dee25442991591842cf4f5e34d8c",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.26-ubuntu-22.04-arm64.tar.gz",
            sha256: "2456cb336de10471199ba38cc680b955242837b49e9b7ef72cf80a63516e834d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.26-ubuntu-22.04-x86_64.tar.gz",
            sha256: "07fba7fb3d730cfebcc6e5fe8ecbc9c799e1d3b207cb3ad251d6579d965cd6b0",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.26-ubuntu-24.04-arm64.tar.gz",
            sha256: "141c7d769728070df7bce8bd1f8cf4e9e43501f55fa4d055750842d2f9da6734",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.26-ubuntu-24.04-x86_64.tar.gz",
            sha256: "ed507c285fd63b0b6ea377b5ae8840dda549a1e03442d95157ca9a5878fe7fa6",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.26-windows-11-arm64.zip",
            sha256: "834bb5d76b42d32840059ee40d0b1ee3c4aa59b03c0882994228a8013c6c6c56",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.26-windows-2022-x86_64.zip",
            sha256: "fc33d7cd4ebb2bc8b7dcacc33ab88d174d07d5b73e4c76eaa22bd74401033b08",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.26-windows-2025-x86_64.zip",
            sha256: "f0778807f16f022f7cebaba7f426d5453ed66313090be031418cf41743208938",
        },
    ],
    load: LoadPolicy::Lazy,
};

/// The `tinyhosts` module, loaded on demand.
pub(crate) const TINYHOSTS: ModuleRecord = ModuleRecord {
    id: "tinyhosts",
    description: "Hosting provider operations",
    bus_name: "ai.tinyhumans.tinyhosts.Hosting",
    object_path: "/ai/tinyhumans/tinyhosts/Hosting",
    version: "0.3.0",
    release_url: "https://github.com/tinyhumansai/tinyhosts/releases/tag/v0.3.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyhosts-0.3.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "368f00ee397649481ff8f61d33eeb8b908cc416d6b2f9d4ef3c81263d6b0460e",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyhosts-0.3.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "e298942fddec8930fd78651632c02abe78968a17ab3589608875b8dc4f731669",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyhosts-0.3.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "f7c9484176b62357fb449eb79830e3df0eebfd4e0814c8d8e1c58669d829f482",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyhosts-0.3.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "576ff6204ccaa1e2dc2ef1b27762848efa4555d4d32ef46b4c329daf1e04eac3",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyhosts-0.3.0-macos-26-arm64.tar.gz",
            sha256: "b0083e70b567659d095a1ff0f9d2c8cb5bafe110d90fb08527c509362ce77a55",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyhosts-0.3.0-macos-26-x86_64.tar.gz",
            sha256: "44fa7c9efde0c388f154f154809ce2829460bae4931f27d20f8f0d3ee3bf2a8c",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyhosts-0.3.0-macos-15-arm64.tar.gz",
            sha256: "56ca93b2cab2010ff599c38179eb99f6ad67631e39f671be973256c3d9142381",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyhosts-0.3.0-macos-15-x86_64.tar.gz",
            sha256: "0ffb9fc8d1c7fc7d95a7808af7722d8c6537a508c0c3a294ecf8fb8a28430207",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyhosts-0.3.0-windows-2025-x86_64.zip",
            sha256: "3d6d5f7fd0806ac6fd6c2ed0c60a9155d6c09a861ebd1ac0c43864e969f88fb9",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyhosts-0.3.0-windows-2022-x86_64.zip",
            sha256: "a5003f1254052828d6fa362b993b0bed6b7256c252f969536090e148378c331f",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyhosts-0.3.0-windows-11-arm64.zip",
            sha256: "21f8909a2a81a541e3d0347510b2bf25baec59438a5536edcd57e65fae7f4a4d",
        },
    ],
    load: LoadPolicy::Lazy,
};
