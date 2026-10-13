//! Registry records for additional first-party TinyBus modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

/// The `tinybox` module, loaded on demand.
#[cfg(feature = "modules")]
pub(crate) const TINYBOX: ModuleRecord = ModuleRecord {
    id: "tinybox",
    description: "Sandbox execution and lifecycle operations through TinyBox",
    bus_name: tinybox_bus::INTERFACE,
    object_path: tinybox_bus::OBJECT_PATH,
    version: "0.1.29",
    release_url: "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.29",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinybox-0.1.29-archlinux-arm64.tar.gz",
            sha256: "0461e2734aac0847ade5392f98369c2aa8ffa03ab85be326a819e2612183c998",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinybox-0.1.29-archlinux-x86_64.tar.gz",
            sha256: "d033d073cad1143578eb3791b9c3dda020d54c83ce228d6278addf3eee5040fb",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinybox-0.1.29-fedora-43-arm64.tar.gz",
            sha256: "6a30bb1d030de6b1a15bedc8d18e2693d6157f4d826af240da8dda0429be0b46",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinybox-0.1.29-fedora-43-x86_64.tar.gz",
            sha256: "5feba5818384b8cccf430ee8e0623eb8137a7ec5c4a5e51d0fbb741f83e59025",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinybox-0.1.29-fedora-44-arm64.tar.gz",
            sha256: "b8befb06f3defb72aafa5b76995723a23ff76c580d96bf79a98ace8fb5b2ab62",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinybox-0.1.29-fedora-44-x86_64.tar.gz",
            sha256: "f3cf809688a4abdd5cbcdc5007762e0c82d625ff7851f0cb06895c162e52573f",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybox-0.1.29-macos-15-arm64.tar.gz",
            sha256: "97f5a6f63001dc0612213e5ce393bb8903db2d6a9cff4be97d90e84afa9dd7c8",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybox-0.1.29-macos-15-x86_64.tar.gz",
            sha256: "45c3a48ebbeb8452bb7bab63d937fab8c70812d831805bc33636970ae94390ee",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybox-0.1.29-macos-26-arm64.tar.gz",
            sha256: "12183a8fcb03ff149f550851c1c4e703a2b092c59a483726b37a501674613ec0",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybox-0.1.29-macos-26-x86_64.tar.gz",
            sha256: "4e74ad646b7fa4c31c619bc9680665b7bc30b06d4f3f42ac769856b195634454",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybox-0.1.29-ubuntu-22.04-arm64.tar.gz",
            sha256: "50bb4b716a7f8f0db62a91a936f4fd4eeee03b96c0649e3e4fc9aa84e563f22c",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybox-0.1.29-ubuntu-22.04-x86_64.tar.gz",
            sha256: "6672b0be315bcebef0eb255255981e034202f59e7223db6a1f8f539e20025a77",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybox-0.1.29-ubuntu-24.04-arm64.tar.gz",
            sha256: "67afe1eed18edf16d832c7d1507bd47fb7dc863d4157edd0e5ecae2afa318b4c",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybox-0.1.29-ubuntu-24.04-x86_64.tar.gz",
            sha256: "a51608744a06e579bc5e3a450521d32de78521162528b1e5005d55585eaf166c",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybox-0.1.29-windows-11-arm64.zip",
            sha256: "5f096509db0a37a5f076d6d039ad4e8b2e3f092f66ab6484ec2f363321cd1932",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybox-0.1.29-windows-2022-x86_64.zip",
            sha256: "1dba24ac470b79246f40fd888c3a62e4a51bab314f0d2d0104ba1ab5d48dd2c3",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybox-0.1.29-windows-2025-x86_64.zip",
            sha256: "89ee0a8503410eee6a1591f730d169dc13102e1cea0873016100dcb35b1835d0",
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
    version: "0.1.30",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.30",
    assets: &[
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.30-macos-15-arm64.tar.gz",
            sha256: "c23e3c099f33121e14a520dfec994e7b5da6205da02ea001d71b28889e5f269b",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.30-macos-15-x86_64.tar.gz",
            sha256: "455e33ffb1d93b0a1023e8b4d528b49ccbc0ed95851b1fb2733649994efe9bfc",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.30-macos-26-arm64.tar.gz",
            sha256: "b2bcc666b980eab9a3090312ed83482a34591f1f4ecc73346f8e1e2422f482eb",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.30-macos-26-x86_64.tar.gz",
            sha256: "d92cb8757f49ebaf3b5d7ecbd79af193a4d4bb002e9b78ce473091d649b08b31",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.30-ubuntu-22.04-arm64.tar.gz",
            sha256: "5852f6888541332cacaed2401ed67815588ba076d60926fdb31daedb13185b12",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.30-ubuntu-22.04-x86_64.tar.gz",
            sha256: "f63bb9034aa48a2335bea0a86ad0f1f920121281b88611c11a9e09fdac8cfe02",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.30-ubuntu-24.04-arm64.tar.gz",
            sha256: "810f68f723e9817d7343944137a00689ef997f57e4e241f496120350a1ba58f6",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.30-ubuntu-24.04-x86_64.tar.gz",
            sha256: "61e7a6f956b17ae53e83a2b0068f1009b0fde0b1266ca4f8f1a7feb8fb077104",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.30-windows-11-arm64.zip",
            sha256: "122b253a859f769a6ae5e89ea9adc5edaefc5471bf26e6bf8d09e7e0bef3d236",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.30-windows-2022-x86_64.zip",
            sha256: "2a8905d6a72bdbc7c4839fa7afb38362deef54eb2296f918b6af016dd8d54090",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.30-windows-2025-x86_64.zip",
            sha256: "feeddcb7beaa48e4bfda78e40cf6ce999cb7e4abd0414a6595da44665546af88",
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
    version: "0.4.0",
    release_url: "https://github.com/tinyhumansai/tinyhosts/releases/tag/v0.4.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyhosts-0.4.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "ffaf1b749808185b01ed36a90d013c8492d03a113cc63eb02657191983c99b1d",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyhosts-0.4.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "10d74c3d50cfe7b099d7ed3cc6017c017f4d807da7622b96482ef0e26371554f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyhosts-0.4.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "cdd9a2b222ec368a20db45c55c3deeb40a4dc8ffc47bea73d16f6edfce864cd7",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyhosts-0.4.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "4b50f878cd7c65812a05384fd824e59c0be9c02e566b472725b04ef7eacdec19",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyhosts-0.4.0-macos-26-arm64.tar.gz",
            sha256: "8452dbbf4de45f313e76e75191923ccdb9d294d2a31dced56b14ee284bbf2176",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyhosts-0.4.0-macos-26-x86_64.tar.gz",
            sha256: "86882b94ee6a3bdb5796523e000136a3780ca59db66a7c645f8a3ea5c2c930b4",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyhosts-0.4.0-macos-15-arm64.tar.gz",
            sha256: "cbb66815cf8513985afde49b3cb9f8406d2bf538609956467da3a7bdacd21044",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyhosts-0.4.0-macos-15-x86_64.tar.gz",
            sha256: "7178b5d5bdb851f4485f8981d99277d160262131c0444996e7a0828209034bda",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyhosts-0.4.0-windows-2025-x86_64.zip",
            sha256: "6a29fc50de7ed2224d6faa26440c87bb0293d7ed0f703bd5cbb169bcb5f20234",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyhosts-0.4.0-windows-2022-x86_64.zip",
            sha256: "730aa387d3f18cdc55f9a3fce83f835a3b8da9a26c0f4701c31c76de790081de",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyhosts-0.4.0-windows-11-arm64.zip",
            sha256: "ab281a73d91117fb12adfc62dfe9e7fbdafa36b5fdfa716c5e440ce33d26a2c5",
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
