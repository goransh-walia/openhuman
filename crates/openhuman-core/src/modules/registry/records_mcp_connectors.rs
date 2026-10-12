//! Registry records for the `tinymcp` and `tinyconnectors` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

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
    bus_name: "ai.tinyhumans.tinymcp.Mcp",
    object_path: "/ai/tinyhumans/tinymcp/Mcp",
    version: "0.14.3",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.14.3",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-rolling-arm64",
            archive: "tinymcp-0.14.3-archlinux-arm64.tar.gz",
            sha256: "ce3429b277e8d6ff18e5cb2eae508681e282c5a662b04be6e5a6b0baef889309",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinymcp-0.14.3-archlinux-x86_64.tar.gz",
            sha256: "e58fa20f6d4f5041951eeb8799050a251a01b5fde1a6442409ebec4ab0387e28",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinymcp-0.14.3-fedora-43-arm64.tar.gz",
            sha256: "d792e3afd6b9bf715450a08b6dc0ea5640b4e31ec05824e73cabca742dc439d7",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinymcp-0.14.3-fedora-43-x86_64.tar.gz",
            sha256: "7d9e36f65058518e58a4a366797786c8c7cd9954070f9a9c2a38f4a4078d6bca",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinymcp-0.14.3-fedora-44-arm64.tar.gz",
            sha256: "ae150372d30a8d90dd82c39a139db36322d03142509a8fa0aaa837227944d315",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinymcp-0.14.3-fedora-44-x86_64.tar.gz",
            sha256: "255559923bd7a2bcf8c7290cfe0d9eb39bde4cfeb99d35bb5ce3dd2c994b1819",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.14.3-macos-15-arm64.tar.gz",
            sha256: "5100539b489a0ea2e6b64334bd3857b6cf63af8d4c0f20d4efba3c675488d8a0",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.14.3-macos-15-x86_64.tar.gz",
            sha256: "d485623e52b5ea1057ff1ad354c2338ee06666ba0ca4e941439e6b0a703e19da",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.14.3-macos-26-arm64.tar.gz",
            sha256: "f147c5fb4220a316e36ce9e68004166faa36b0f31d96f89cf154867b2d1cd13c",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.14.3-macos-26-x86_64.tar.gz",
            sha256: "fb07cecdcf1c78076de4e985c089c17a3c7a78e2d3c7f35882eec5b8612c2418",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.14.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "fc0b6c92683abf8b8d3c78983ce325d720482d23ecde60aaa606c5e6e0952e3b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.14.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "e7f2eccefbc645c66916c4885658a1dc286f1eb72ec7a7546bbc9f37e4228946",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.14.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "f1cd87ed2a3276a5e8bd483ca134ffb12c0638d94701be83dd54ddbab3f64c51",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.14.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "4858b6eb8d246ae3f18f0e3e5de02bf98f98c7cd9ca3489d2049200b09abf15e",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.14.3-windows-11-arm64.zip",
            sha256: "0ab17f2ceb5134984b13746f2c8ac2e04fa632d66faa691d61019d08f1470adc",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.14.3-windows-2022-x86_64.zip",
            sha256: "891e9be9218e4e67a6c6ce64f2ca593bd98dc13fb2885a5d2c5d1c787b371fe7",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.14.3-windows-2025-x86_64.zip",
            sha256: "82e6e42cd586bf5a2dcec7b56fe88b158e5078f4e26e55dd4154ef70467e89d2",
        },
    ],
    load: LoadPolicy::Lazy,
};

pub(crate) const TINYCONNECTORS: ModuleRecord = ModuleRecord {
    id: "tinyconnectors",
    description: "OAuth connector integrations: accounts, actions, and triggers",
    bus_name: "ai.tinyhumans.connectors.Composio",
    object_path: "/ai/tinyhumans/connectors/Composio",
    version: "0.14.0",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.14.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.14.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "836bffa2fe4f935c8504f608e08bdf023326d1285fc977fd37149f2799899478",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.14.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "b7dbafe019aca715e55444a37b5ff4fc469ef2bcc675dfa987e1de626e70fd22",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.14.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "a29cb970b385cf8cd40998baa3b6f040447196d019a8d3413d67ca64a6509687",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.14.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "e961477b9cac1892beef8b498ba4f44daa4b39b8dddf277ec49496e3869427b5",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.14.0-macos-26-arm64.tar.gz",
            sha256: "5a24f1eabb0f246a5954e4d6f93cc0feeaed4fe6bfbf1fceb7399078e0e5734a",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.14.0-macos-26-x86_64.tar.gz",
            sha256: "5df7dbc74b2f39174e57695c5556ab4c232b775897e7bd534be015ead01ef51b",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.14.0-macos-15-arm64.tar.gz",
            sha256: "ea8010163f279f14e25a4ad884b4665e24385c560dd2caf46e8c80220556ba11",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.14.0-macos-15-x86_64.tar.gz",
            sha256: "508e44cc9cd273dcbd2d269295ab0b7009efb1a27d3244ba9adde6b297306db1",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.14.0-windows-2025-x86_64.zip",
            sha256: "5a5e62e8d11bbdc792c9d8fd2a9b59bcea9add80978457ea0faa7c058cd48546",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.14.0-windows-2022-x86_64.zip",
            sha256: "1560ed909f338a5bf1118d644d44e1fbeee5c26461241df9f24630e3f9ba3cbe",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.14.0-windows-11-arm64.zip",
            sha256: "0bdfb0424ad62fc93ba66a7d8dd7063a39ae8656b78d2e65f4f1a6425aec3279",
        },
    ],
    // Lazy: a user with no connected accounts should not pay to load it, and
    // most sessions never touch a connector. Safe even signed out — the module
    // loads without configuration and still answers the capability members.
    load: LoadPolicy::Lazy,
};
