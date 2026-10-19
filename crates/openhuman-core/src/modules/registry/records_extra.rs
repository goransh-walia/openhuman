//! Registry records for additional first-party TinyBus modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

/// The `tinybox` module, loaded on demand.
#[cfg(feature = "modules")]
pub(crate) const TINYBOX: ModuleRecord = ModuleRecord {
    id: "tinybox",
    description: "Sandbox execution and lifecycle operations through TinyBox",
    bus_name: tinybox_bus::INTERFACE,
    object_path: tinybox_bus::OBJECT_PATH,
    version: "0.1.39",
    release_url: "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.39",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinybox-0.1.39-archlinux-arm64.tar.gz",
            sha256: "0e396ace74f3583c15a236a305a3fabfb30b04b0b9e049540c53fd20d74f3397",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinybox-0.1.39-archlinux-x86_64.tar.gz",
            sha256: "2f7694535c2d12792f2b1b8bcb55a0619366c5e2811832b48ce7dc29cc9e065c",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinybox-0.1.39-fedora-43-arm64.tar.gz",
            sha256: "b86594c4bf2c1983ec5787b92e8257d7471d3657152fd7079b2a8daca1a01443",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinybox-0.1.39-fedora-43-x86_64.tar.gz",
            sha256: "b7fd2d33238401e2a9713cb99641cd4e3a318919ead3c2354549d0980002795d",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinybox-0.1.39-fedora-44-arm64.tar.gz",
            sha256: "ae84b73d7710c8badd1218b331403a1a859c6b1e0dcca0373f4d1db2638a1985",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinybox-0.1.39-fedora-44-x86_64.tar.gz",
            sha256: "19b4c501bc0f24e24f2cee5f3ab201d25ea5b6188fb939805330d066f5ea1ab7",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinybox-0.1.39-macos-15-arm64.tar.gz",
            sha256: "fba75a5b4b7297eaca671ff599bc0d566b43416e7e65f2105e118b7a298712be",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinybox-0.1.39-macos-15-x86_64.tar.gz",
            sha256: "adcfca25570a4a20a6805ae709cfedada6767250a58f2d70659701a457b28929",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinybox-0.1.39-macos-26-arm64.tar.gz",
            sha256: "617a90356fb4717b766dfe50bbfb6f6e1da8fd2e97c041fddf017e0e72c04d6a",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinybox-0.1.39-macos-26-x86_64.tar.gz",
            sha256: "ce57f62396e4f8c6e54b59cf17da1e975189780e96490242b7c80cbb881d5722",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinybox-0.1.39-ubuntu-22.04-arm64.tar.gz",
            sha256: "b19c6b50221c140d95a071815111d28417161c52a1f4a2ebaef02212326e8058",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinybox-0.1.39-ubuntu-22.04-x86_64.tar.gz",
            sha256: "70ed28886577abb0906f7ef418592fea7804b3f4b42477679258481c3f1479e7",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinybox-0.1.39-ubuntu-24.04-arm64.tar.gz",
            sha256: "79b38dbb87f4f611f506aac9df204d9c3eb79e5e2e86e70f4aeaaad11861cc52",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinybox-0.1.39-ubuntu-24.04-x86_64.tar.gz",
            sha256: "05f298a22db26616059be4daf3f1c92a7e76b0735a8ceb912ac0007de6cb3cc4",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinybox-0.1.39-windows-11-arm64.zip",
            sha256: "5ae0085187208025c9281b5c0ea1dfe6416f9484312ad2e2130baea51ce2e6bd",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinybox-0.1.39-windows-2022-x86_64.zip",
            sha256: "20876bc82faf374e7752c188ed89f7ca2e45cc6cc588130962cceecd202ac637",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinybox-0.1.39-windows-2025-x86_64.zip",
            sha256: "fe8f0cc2204ce3fa4af9de59fb390325a0a36411c1af14c1cb367e95bb788b9d",
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
    version: "0.1.36",
    release_url: "https://github.com/tinyhumansai/tinychannels/releases/tag/v0.1.36",
    assets: &[
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinychannels-module-0.1.36-macos-15-arm64.tar.gz",
            sha256: "580c5ebf769b2a72130378f894490a635d180429b0b54326710bcef18d3f0e07",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinychannels-module-0.1.36-macos-15-x86_64.tar.gz",
            sha256: "77c2f35f027a3d040786dbf4fd760e7359fb007dda7ef2793915cb5447d63e9a",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinychannels-module-0.1.36-macos-26-arm64.tar.gz",
            sha256: "91d5dbdd181b909769c6ea7d72d7b93210842da6af7ea8ce2802264a5678bd7e",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinychannels-module-0.1.36-macos-26-x86_64.tar.gz",
            sha256: "792439fdf93c9d6c90cbd58c66b37ba7f8cf0130bf4184258675e096890b7f73",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinychannels-module-0.1.36-ubuntu-22.04-arm64.tar.gz",
            sha256: "a9c166f223096a6a89dfb395fd88c1a62960e2d174cc6a2246f82fb5a21f307c",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinychannels-module-0.1.36-ubuntu-22.04-x86_64.tar.gz",
            sha256: "5c510f268c794eda3897c7d77df8ca2223d70df9147ae594d6bcc9258c090d96",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinychannels-module-0.1.36-ubuntu-24.04-arm64.tar.gz",
            sha256: "79bfd2fe7f53504169ab5c62afb9c6c7dc8ff4f183bbd150a673f1c4d7c7612a",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinychannels-module-0.1.36-ubuntu-24.04-x86_64.tar.gz",
            sha256: "ae50f7234edb5133db96de63b6918e0b602a269fdcd25620ccd1a4d24eaf1a98",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinychannels-module-0.1.36-windows-11-arm64.zip",
            sha256: "c0449841d4f261612485e58c73cacc4dcdf024eebda6c921eda4f56b75ed0ba6",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinychannels-module-0.1.36-windows-2022-x86_64.zip",
            sha256: "9e23a4806f20ac877d1bc2e2c55e77b701011eade707859a26c2d9412136ac03",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinychannels-module-0.1.36-windows-2025-x86_64.zip",
            sha256: "6b39da8c6e756b89596ba9342f3d708bc0e0f38ee56381b77209f6d38676a178",
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
