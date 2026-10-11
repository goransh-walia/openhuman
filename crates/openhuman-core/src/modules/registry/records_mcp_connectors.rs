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
    version: "0.8.1",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.8.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.8.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "54e05304cc088f83aeb858780426d06aebe4ed604c8a92d8d66136c532807c21",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.8.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "78bfa9a7e6751b6f7b7c2d6242f6e5f9369421e4ff54e5aef990f1324a386325",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.8.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "eb190c7af70801d17592027dba60c4669e804244b3111859444ca641c209a588",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.8.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "59dffc24b69552854bc9c6dd1f2d4c07293d15e44d3a4de44a86387b1afb0725",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.8.1-macos-26-arm64.tar.gz",
            sha256: "05df5adc80296ba4a6882d5aa519fcdbd7ccd8671ee94ad8b647268bc0cd0523",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.8.1-macos-26-x86_64.tar.gz",
            sha256: "9e4047d0fc1b54f9ec9aa531a604771e2037390a5178413e652248a9e64eef91",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.8.1-macos-15-arm64.tar.gz",
            sha256: "cdf409f7e2e93e9d75f35d8f4e7186ddd090a583a48eddf33ef62b1d83c2af89",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.8.1-macos-15-x86_64.tar.gz",
            sha256: "0bc9156d0c2bcf9ba6e9277ee9ad3846185dc2192624f5989ab1331ed12ddaf6",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.8.1-windows-2025-x86_64.zip",
            sha256: "f2a32c36fe35c7d24e1bcccf489a686e01c3fb875ec10662be45c2858a24e280",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.8.1-windows-2022-x86_64.zip",
            sha256: "e9b14fd191d2269bb6cc088c76e3ff96a8d97197be9554b0ac8fddd9365c92cc",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.8.1-windows-11-arm64.zip",
            sha256: "6fe315376f651a9f425c437d76f4bf50f0a8838d783a94d0fecf61fb2adb71a9",
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
