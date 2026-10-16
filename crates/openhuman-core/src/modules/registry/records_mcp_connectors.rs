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
    version: "0.18.7",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.18.7",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-rolling-arm64",
            archive: "tinymcp-0.18.7-archlinux-arm64.tar.gz",
            sha256: "3998a2a151419a277c2806918c46d06b74408cbfa17bbbd84559a7abdf45eb85",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinymcp-0.18.7-archlinux-x86_64.tar.gz",
            sha256: "892c4a3bb6082413725ef7807097d97f2e4ed32bdf2d7bca26c9effdbb8daead",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinymcp-0.18.7-fedora-43-arm64.tar.gz",
            sha256: "e80ab9613921ab6d2534c9f4701452b4ea60fd8842538eab7e15cb7a13be6fea",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinymcp-0.18.7-fedora-43-x86_64.tar.gz",
            sha256: "9edbbb273c6601c8ada3229c08863397582400243ed9086d4fa57467729b1d2f",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinymcp-0.18.7-fedora-44-arm64.tar.gz",
            sha256: "591f09b0b22d4d83347571d8e81fccacf009b4ad3c65d41cc074f7352990a3b5",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinymcp-0.18.7-fedora-44-x86_64.tar.gz",
            sha256: "60ba90075e280147c58f40963df8aa02388f08cc515da5277362ec1eca6e17b2",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.18.7-macos-15-arm64.tar.gz",
            sha256: "d96b83a21373d7494a586d4919df9ba0e62a07a00858495b802876cc55bc6dde",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.18.7-macos-15-x86_64.tar.gz",
            sha256: "6b623bbafcc5cf87fa37333e060a511feef3d2e2912207ccc648a7c9b36a85cb",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.18.7-macos-26-arm64.tar.gz",
            sha256: "26f26aaf8ae7ec1d493dfae7d5ad976ef3cc21774e92a8ae0266611eb0ac7549",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.18.7-macos-26-x86_64.tar.gz",
            sha256: "47b4ee4f424f17d336a8ea6669fb15be747895b71afe94e3de06ec104e782ee0",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.18.7-ubuntu-22.04-arm64.tar.gz",
            sha256: "ff09f728ccbd0a7b4a47ea65b2ce6b0328fc4d3f77a37cd397af5847f8fabc4c",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.18.7-ubuntu-22.04-x86_64.tar.gz",
            sha256: "1111e4f5e8bb952756364315c759dadf07fd4a84aa9f7fc44866c1f15b39c5d8",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.18.7-ubuntu-24.04-arm64.tar.gz",
            sha256: "55a8e893f28a817e80d00c4583bc13d7167529595a497c5179278badff5a1e33",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.18.7-ubuntu-24.04-x86_64.tar.gz",
            sha256: "89841ecda7285bab1607a884202b0ac803f9ce1faf009168d9cd851e7e390b25",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.18.7-windows-11-arm64.zip",
            sha256: "2730c2aa874877fec56c72be96aa796f4d7da2d5b3b41a5f4cb1610fbf7a5fbb",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.18.7-windows-2022-x86_64.zip",
            sha256: "87630195eb20fdb8b337128c4552cc53a555168409e1dc3eb3865c9e04df8b9f",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.18.7-windows-2025-x86_64.zip",
            sha256: "f051c294f9c366905e1852e20b2e10d6fcf0b21839e815e31d8368fd9e1ffdcb",
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
    version: "0.19.3",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.19.3",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinyconnectors-0.19.3-archlinux-arm64.tar.gz",
            sha256: "1b223604c366769a67d0345d12da9b1b679722b54062157c68fba08b608f3ec9",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinyconnectors-0.19.3-archlinux-x86_64.tar.gz",
            sha256: "b7c83dbf1a4f40a343fe687708412f5b33c137cf384ebea5d8b9b6223d890f58",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinyconnectors-0.19.3-fedora-43-arm64.tar.gz",
            sha256: "386552936cc9964ec4a006664656fa1ac58aaccd104dbf8fd71626824805dac4",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinyconnectors-0.19.3-fedora-43-x86_64.tar.gz",
            sha256: "7d810750a28d64ff332448095c91e57064da1dfdd70028ac378c86c88df8ca2c",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinyconnectors-0.19.3-fedora-44-arm64.tar.gz",
            sha256: "a7094a70e28f99dbe4312be4c0d5b49337b5b35e605bd07c7dde323e247fcdf6",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinyconnectors-0.19.3-fedora-44-x86_64.tar.gz",
            sha256: "39c4938e9daba8643910e2b4f8573c0559d1cf8a15b44bd99e615a9dab926081",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.19.3-macos-15-arm64.tar.gz",
            sha256: "644a1c58c2ab7911ea2808c23b75b96e3ae312af1d12f97cca55f71971f551b9",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.19.3-macos-15-x86_64.tar.gz",
            sha256: "53297c0fffad47ac19dba05a9e360af1601cf4de779e7db0d9e06d2059e91099",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.19.3-macos-26-arm64.tar.gz",
            sha256: "96290a9f9b9b07fffbe3f333478002d7f46e5c2a563542d73ddac47420e597a4",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.19.3-macos-26-x86_64.tar.gz",
            sha256: "990a0414fb4a2fc834efeedcd5e4e51e7057aed928f9c51aa4fae2c4a43a7f0d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.19.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "f5ab8ced9095150df3ab0f3b84b4fe505a61d1fec64de535355d59a7ee13954f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.19.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "26daaa208a624c1afc5520fdd3d715eca032b956324ad5918431171097381620",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.19.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "ded63f30313e3196c0f75e6c6d211920d799df5e39b4d0657c4901de419db96e",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.19.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "15cfa5718aeabe2882503c618e22c5067f050fbc3aaa4be31e541071700900f5",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.19.3-windows-11-arm64.zip",
            sha256: "02a7e82e452c9190db68b5f2eef49b6fad73bf067dbb8f13ab3688c003dedf61",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.19.3-windows-2022-x86_64.zip",
            sha256: "cf0d99f72732d934b24428cfa2ef414a649d41c612de330ea76af4e037641822",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.19.3-windows-2025-x86_64.zip",
            sha256: "271bd3855a1565fedd041a02bbca2555d418bcd16d764b55385f6896d8e9682b",
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
