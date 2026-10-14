//! Registry records for the `tinymcp` and `tinyconnectors` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

/// The `tinymcp` module: the Model Context Protocol client.
///
/// Owns both transports (Streamable HTTP and a subprocess over stdio), the
/// statically declared server set a host puts in its own configuration, the
/// dynamic registry of user-installed servers with its SQLite store, the
/// reconnect supervisor, the browser sign-in flow, and the write-audit log.
///
/// Lazy, because dialing an MCP server is something most sessions never do: a
/// host with no installed servers and no configured ones would otherwise pay a
/// download and a `dlopen` for a capability it never reaches. That differs from
/// the module's own `lazy = false` export hint, which speaks for a host whose
/// servers should be connected the moment it comes up — this host decides when
/// that moment is, and does so on the first ask.
///
/// **What stays out of the module is host policy**, and the split is the same
/// one the contract's own documentation draws: the prompt-injection scan over
/// remote tool definitions, the `mcp_clients` RPC surface, the
/// agent-facing tools, and the proxy *scoping* decision all belong to this
/// application's threat model, not to a protocol client. `tinymcp-bus` carries
/// the vocabulary; this table says which bytes may speak it.
pub(crate) const TINYMCP: ModuleRecord = ModuleRecord {
    id: "tinymcp",
    description: "Model Context Protocol client: transports, registry, and the write-audit log",
    bus_name: tinymcp_bus::names::INTERFACE,
    object_path: tinymcp_bus::names::OBJECT_PATH,
    version: "0.18.5",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.18.5",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-rolling-arm64",
            archive: "tinymcp-0.18.5-archlinux-arm64.tar.gz",
            sha256: "0a3760807e225282677caba6a157e5c6cc16cfe509d22a8acc372a9a4c77f9d1",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinymcp-0.18.5-archlinux-x86_64.tar.gz",
            sha256: "84f3bb050943bc040204b316c3540e31b25457b4f36112f0248f3a2bbd3f85c5",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinymcp-0.18.5-fedora-43-arm64.tar.gz",
            sha256: "fae7790e4df4418eb8fff18818f5df04d69e2c307d3613b0fc177ad0d1810804",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinymcp-0.18.5-fedora-43-x86_64.tar.gz",
            sha256: "b8fe422717db38f63619bd108b2b63e9a3f3d40e8cb2ab653076d76d55f8bcf8",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinymcp-0.18.5-fedora-44-arm64.tar.gz",
            sha256: "cc3f6cf33631e28921dd4483dd390988ab9a86806b0ce26274fc92fe0c56c53c",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinymcp-0.18.5-fedora-44-x86_64.tar.gz",
            sha256: "f92f83766f9f277ad8a8c4c9844cda5b1bcabc6404d8a133153cfeb941454f87",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.18.5-macos-15-arm64.tar.gz",
            sha256: "c706d86b21edcd0bfd6a9b01ac055f475904ba82a003be4dbdd8c715237901b9",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.18.5-macos-15-x86_64.tar.gz",
            sha256: "b675c9777403c7215ce850ec79cdf3607d244631a0dd20eca7be4967455e548e",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.18.5-macos-26-arm64.tar.gz",
            sha256: "1296c0c405553eb30e4682dc304e657ebab2b7856a49f684e5a7aac6a287658b",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.18.5-macos-26-x86_64.tar.gz",
            sha256: "4c34fb291190b4317ecf895869c38baa14689693e89d9af344642a222d683157",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.18.5-ubuntu-22.04-arm64.tar.gz",
            sha256: "404ec212c3b3143308ac7f892f558fbd1440e863e1f5a7cfeeb19d54c2bfcfad",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.18.5-ubuntu-22.04-x86_64.tar.gz",
            sha256: "bf569fb43def4f743c420701a4e9e39fc64679e1dceadac9e87894135d1b5013",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.18.5-ubuntu-24.04-arm64.tar.gz",
            sha256: "b1525e225a1481600624bd317c01cb2c13ccc4034860103fe775ff6ecda40803",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.18.5-ubuntu-24.04-x86_64.tar.gz",
            sha256: "f8c0b9dfc9a3a4b98314f5acf298c61b5ef96b3ce8605bad923a6bd642aefde6",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.18.5-windows-11-arm64.zip",
            sha256: "305bb0401e92752672c4db1eecedadf2c6bb720a868ab2316f97287048081b49",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.18.5-windows-2022-x86_64.zip",
            sha256: "6457801f2919cfe5e47dfe3577f3f163ad3f7567a814f99e51a37dee254f9c86",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.18.5-windows-2025-x86_64.zip",
            sha256: "2c2e2487c6b3fff2975df7b5928e688c46e42b9260afef02f72a0a77e1bc118a",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinymcp",
        manifest: "crates/tinymcp/Cargo.toml",
        package: "tinymcp",
        lib_name: "tinymcp",
        features: &[],
    },
};

pub(crate) const TINYCONNECTORS: ModuleRecord = ModuleRecord {
    id: "tinyconnectors",
    description: "OAuth connector integrations: accounts, actions, and triggers",
    bus_name: tinyconnectors_bus::names::INTERFACE,
    object_path: tinyconnectors_bus::names::OBJECT_PATH,
    version: "0.19.2",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.19.2",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinyconnectors-0.19.2-archlinux-arm64.tar.gz",
            sha256: "b47376cfce7b8e9f72848eea420fdfb39de2a224a4093bac9134f2b29a29252a",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinyconnectors-0.19.2-archlinux-x86_64.tar.gz",
            sha256: "ffe6f0fefff0ac1d0d2495d10f59538b318ccaea63f0843aa0d85398e0954751",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinyconnectors-0.19.2-fedora-43-arm64.tar.gz",
            sha256: "69a2198c10a4691874f2207d24f6ebc24ccd35fc6b35c2fef3350f1646d92977",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinyconnectors-0.19.2-fedora-43-x86_64.tar.gz",
            sha256: "47f7645663333338541e5f315809e9d2e56eec7e50146112198f5e029052058d",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinyconnectors-0.19.2-fedora-44-arm64.tar.gz",
            sha256: "061521ffec706e498087b111147f51140466c5bfbf4869af396c5bb1266da3fb",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinyconnectors-0.19.2-fedora-44-x86_64.tar.gz",
            sha256: "8d0eb0c66fc93edfbb10718196e8f92309348004c42cddcc3ac20df5b3097f2f",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.19.2-macos-15-arm64.tar.gz",
            sha256: "8de84e0d0fd802a890fe4c6d9cc2cbdf3dae5b1c83d31bdc72a8dc22ae32ccd7",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.19.2-macos-15-x86_64.tar.gz",
            sha256: "df3e1075fbfff0d2b8b51e9ceb49d993c212f97e6359038aaee03ece683a09e9",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.19.2-macos-26-arm64.tar.gz",
            sha256: "a62b4687d9cd8fc3580d04fa6114f24c14e133b32aa11edf3ad8df386ed68c75",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.19.2-macos-26-x86_64.tar.gz",
            sha256: "5fd930cc6d3a760965a5377b238b2b466d1060c7ac0c544d5e09613b8705e060",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.19.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "9bffe56deb7a094245bfde56a6fe97ab26a8cab123a1afe2efefda50302ebf0f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.19.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "d22cdb177ff84af72fc19ca55ceab5f4fe7380389061811ee93fe01410a05b0d",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.19.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "f5ebc74a9f326990123884c190528cab0e7e80f26ebb138b4d0f78d2882b1006",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.19.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "9d4e7d255197e7ec4e41e4fc68d53714dad3b4bc3dd17381bdd779d7e7454d48",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.19.2-windows-11-arm64.zip",
            sha256: "bbc136ad5800739a171b01b2eb578c5102c4cedc40de849172b77c0f0929a38c",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.19.2-windows-2022-x86_64.zip",
            sha256: "10a3412df6549fb3e8ecc7f69703509bfacb6679b6c82b28fa0227ea8e09a145",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.19.2-windows-2025-x86_64.zip",
            sha256: "ff4f7b4c68bc2799695ab8990673bcfd1124a43ad2f43dfdba57a06763c2f83a",
        },
    ],
    // Lazy: a user with no connected accounts should not pay to load it, and
    // most sessions never touch a connector. Safe even signed out — the module
    // loads without configuration and still answers the capability members.
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinyconnectors",
        manifest: "crates/tinyconnectors/Cargo.toml",
        package: "tinyconnectors",
        lib_name: "tinyconnectors",
        features: &[],
    },
};
