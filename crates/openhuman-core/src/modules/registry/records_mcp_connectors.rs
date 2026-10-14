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
    version: "0.18.3",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.18.3",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-rolling-arm64",
            archive: "tinymcp-0.18.3-archlinux-arm64.tar.gz",
            sha256: "de9f214fb71a66fbc6ba9d7513af2d41441a285627c39a01e94e2ca82d7f18fd",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinymcp-0.18.3-archlinux-x86_64.tar.gz",
            sha256: "fa5f1829bb3cc866b9d93de522d275ff19cb83c38d44563f968633eb840ed0ed",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinymcp-0.18.3-fedora-43-arm64.tar.gz",
            sha256: "7238a7eeb8828742f34fb0584ddd7ff9b3243f0e08a083849a2825a4babf2bd8",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinymcp-0.18.3-fedora-43-x86_64.tar.gz",
            sha256: "0ac31264b19d93d5a5a43893c9fd6af08c61bc87edc0309928cb11758911e231",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinymcp-0.18.3-fedora-44-arm64.tar.gz",
            sha256: "1c388963a6440edbe5f19100c54b454b85f10e8d586a723131800375ecb64681",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinymcp-0.18.3-fedora-44-x86_64.tar.gz",
            sha256: "14c3fb5e0ef6bc98a35000a8bf628718347df52a40c116f3f76305c09e117768",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.18.3-macos-15-arm64.tar.gz",
            sha256: "6bdda01a7a74411591641f9e321a712b329fe1a4cf77e724386f8ac6b3cab79b",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.18.3-macos-15-x86_64.tar.gz",
            sha256: "ffe5efb4148be1658e9fef091df86cdd7f6378dd54dd0bd9e41b70b564a659a9",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.18.3-macos-26-arm64.tar.gz",
            sha256: "62e74c411fa3d86638ea157c24efb8cbce0cef44b13862b3654150292186e0f5",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.18.3-macos-26-x86_64.tar.gz",
            sha256: "625b0c4ea61c68c64aae80ecdcb73a1993d4a27dcf371c939ce58b879fc8558d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.18.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "02cc3f971c8e1e5225b34f64a8602c004f40aaede0ffa6fa7eb2f7cc1b6ebd48",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.18.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "388bda6840e622b6f0035b829a4f043400d2a0d611085390c2e3ff3bc07f0dab",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.18.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "696294c34bf323b1a4723dd38548494de4096bc3c7b134a66d7e28cb67a1bc90",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.18.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "5448a1ca48e8409679900e704c161d411ef70754d3a1fd3f623540f6144be1c3",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.18.3-windows-11-arm64.zip",
            sha256: "8ce944ed35571d765059cac220c52fe8c81c87a1d5acd41fdfc935562463a10d",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.18.3-windows-2022-x86_64.zip",
            sha256: "1ed99a67ac1caf0bc492d39b2f98e2f4c55bdfc7165cdb96352defd9a2104936",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.18.3-windows-2025-x86_64.zip",
            sha256: "a0123aef817aef2e1ad5725fe989d6b0e49ebeefc0d32db1b31a7e43982d84c1",
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
