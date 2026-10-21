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
    version: "0.20.0",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.20.0",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-rolling-arm64",
            archive: "tinymcp-0.20.0-archlinux-arm64.tar.gz",
            sha256: "553aff39b1980d26382950ffda719bd64ecef7416e148c7367c58cd869f95785",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinymcp-0.20.0-archlinux-x86_64.tar.gz",
            sha256: "88eda15faa5be1b5fc3b337e30a10798e846dcc07e4255b131f3c18154c10c26",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinymcp-0.20.0-fedora-43-arm64.tar.gz",
            sha256: "89aeb98105e462ab4ecad395eac54644e3435b378e6e294d8a03e877d0e24f2f",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinymcp-0.20.0-fedora-43-x86_64.tar.gz",
            sha256: "a8187a7737ca14e0c35e3d59a5ef8abfc40db130ce386bf0c709951c93206e91",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinymcp-0.20.0-fedora-44-arm64.tar.gz",
            sha256: "741416ed3739be431f54b65e422a88f7cd73c67ea27b22e3e1d19ce644d0c556",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinymcp-0.20.0-fedora-44-x86_64.tar.gz",
            sha256: "dc6a7f664f7a995746a7f50f5ed0c3f4ef58d3e257c55e996a0479ce5581d5b4",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.20.0-macos-15-arm64.tar.gz",
            sha256: "3267b60a48c96ac3466942ebc6ffcf9c239a5c9913bf57c50348e46fd2cc5a16",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.20.0-macos-15-x86_64.tar.gz",
            sha256: "ccc01999dba5b9b87c1cbaacb6e6e5569d518150410cf2177e272cd4d639dbeb",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.20.0-macos-26-arm64.tar.gz",
            sha256: "8939493522bedc2c7f667686bf6f839dce58b24595d3c03b8077868e714bee40",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.20.0-macos-26-x86_64.tar.gz",
            sha256: "c5307a71d7f1d0e9b34aeda3b90f55361e673fb5f67e36be2597d3a1dbd160fb",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.20.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "4d89f2207f05d12f957b978ec3b326e5f4310e3c8ef61315492582b9e17b8181",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.20.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "86dec3ee58d01f58c82c4213237196060fc7d05370d8e293b77d71d7d88c0334",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.20.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "ffd648c2fdf9d1683923b14b5076b0513d31c4e05afa6fda9dcea7c70e5cae6f",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.20.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "b432bf96daa177ec2562a70ce11ff0616924931db141a58f0d4d24f988b3c94d",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.20.0-windows-11-arm64.zip",
            sha256: "ef0a831e9c8223f3d9e541ca9d020f835c71290fcde6f6668c3cdf2d72b24224",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.20.0-windows-2022-x86_64.zip",
            sha256: "6032d1a08865d917e203ef0cf4ed9f86460007200c6d2649d3604e3db5f6bd6d",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.20.0-windows-2025-x86_64.zip",
            sha256: "6669bba3014b067249abec1505b0dec057aced64c3b109406df8b10fb873f70b",
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
    version: "0.21.0",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.21.0",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinyconnectors-0.21.0-archlinux-arm64.tar.gz",
            sha256: "d147513762e370f683b47f50c58ec9b68b7a62e19ed15759acc854c3e24da3ad",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinyconnectors-0.21.0-archlinux-x86_64.tar.gz",
            sha256: "1672f0299d80b7436dfb485ded0d0db8734482a62bae699179336b3267557ee4",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinyconnectors-0.21.0-fedora-43-arm64.tar.gz",
            sha256: "7993d561bc350d67ab60f8f5c31ed2705aca376fcd34c5a6c61cf127b63b76ac",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinyconnectors-0.21.0-fedora-43-x86_64.tar.gz",
            sha256: "5fd3b94e49d0d45fb4e2a92a1a668a7a0d5cf8d3a08f6843c6381353f4209e5c",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinyconnectors-0.21.0-fedora-44-arm64.tar.gz",
            sha256: "003b09f5d71c170e72077f046fff429f8c8b61fe6031668b61fc736a22c37457",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinyconnectors-0.21.0-fedora-44-x86_64.tar.gz",
            sha256: "d95a6bc885d78149bb5d26dce4b291a46b6ffd75d2ce8c6279ff36266448f7ce",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.21.0-macos-15-arm64.tar.gz",
            sha256: "cf6135caf054d40a5af5b001cc9e549fbe63b1a05f65482d482025bc01cdf2a7",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.21.0-macos-15-x86_64.tar.gz",
            sha256: "e8fff9402d4ba9453d7cf518e8bea94edae49da8709279e8aa78b432009e0116",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.21.0-macos-26-arm64.tar.gz",
            sha256: "ca835cbb88e12de0678cc85be3a99cc6e42e415fa73a251c3b5afc344aee4a1e",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.21.0-macos-26-x86_64.tar.gz",
            sha256: "59060a2f3c7aa71363f7c30b618d72b7181179459ba349f8ec0a9fa536919f44",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.21.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "f9cc5b782436f3301fe06d62aed87c479ebd4c6745f0b26ea138825d5904f0f9",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.21.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "f1210cc9ae65078d7f2604a4dc5dbe57a53b49a1e5017dee5d6e14e57b02de64",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.21.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "99e8ef19397aa04684d1948b46d94eb94c5fa192ffbf3e47cbe6aedd1fd069b1",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.21.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "cfb40dec770adb4a4b2d2db4161f3301f8a14fe8fa7dbb0683b6d579ff2ed2fe",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.21.0-windows-11-arm64.zip",
            sha256: "49f26d501dce7bee9191f594d782cc4f5f7ff6d0ba9e710ab6413194fdc0c371",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.21.0-windows-2022-x86_64.zip",
            sha256: "3e96e044e6a310c8e97c2c598198db79b26dca5cee597a7aeb51097d4be119ae",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.21.0-windows-2025-x86_64.zip",
            sha256: "d7bfe7141ebedfe406db16fc0df0d93f6f658cb0cdbad0415b6c1f332c5167b7",
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
