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
    version: "0.20.1",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.20.1",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-rolling-arm64",
            archive: "tinymcp-0.20.1-archlinux-arm64.tar.gz",
            sha256: "d902e2ac55e9ef1cb0e88dbcccdf22d2f544e0097517331b85e561664f1087aa",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinymcp-0.20.1-archlinux-x86_64.tar.gz",
            sha256: "7530448fc721425da43e795b8bd1a9d679cc04fa8eb5d48363db92cc3916f37d",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinymcp-0.20.1-fedora-43-arm64.tar.gz",
            sha256: "d88a6951ac5544afc136326e4756503c42f17038023a11684c67af74e56a6593",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinymcp-0.20.1-fedora-43-x86_64.tar.gz",
            sha256: "f410705d507e40686019f4fb35a2c254b5f5789da18aa2c3b287f1272a1228e6",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinymcp-0.20.1-fedora-44-arm64.tar.gz",
            sha256: "3b58fd99d1062adad06c64d8c4ade1bef316ffaae3076b49cf312db82f5d8ffa",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinymcp-0.20.1-fedora-44-x86_64.tar.gz",
            sha256: "9ddba2d03dc08774fc9eb84a483c752499ea10b29e63b658ed45a21cd4c6fcb0",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.20.1-macos-15-arm64.tar.gz",
            sha256: "9f2794db8966cd1e9b04d3c9f0e00bb57646e094394ec468d23aa4e6f9c83a1e",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.20.1-macos-15-x86_64.tar.gz",
            sha256: "e66f2c4419b9a407db3fb1a1feb906bb9e90259748e69addc6b815a8f97fab88",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.20.1-macos-26-arm64.tar.gz",
            sha256: "7a19543c53bef5f2b955b49216f9679418c7f9855870db281616991493c27408",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.20.1-macos-26-x86_64.tar.gz",
            sha256: "dea4e31e951173a7a41e0ed6abe4f03aac894f70767d264a573f0d74cda7bf59",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.20.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "e96f44729fa596ecafa42f99c7405963a652bc9862a38414eb9a0e2d9a774031",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.20.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "73746b2459c93129c6ad75cae79e0940d60b8b20e153c9ddb5dba8bd39d24412",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.20.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "73e66fb2cb1460ea5b08ff9e9e70fae2d8f7e2fe2cc837adbd4e88e3e07290df",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.20.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "edc924e1ade72ca3c6e193048302a5f0aaadde10554826b95ddb01247c83ca2b",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.20.1-windows-11-arm64.zip",
            sha256: "839c7ab05574d544769bb8b1042d8313edf755da89484f2086db36e3095fc397",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.20.1-windows-2022-x86_64.zip",
            sha256: "4a3012be1c2c19a5f5c105e4f8cea23f6e5a636aa582322b7eba4e01b90cd26e",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.20.1-windows-2025-x86_64.zip",
            sha256: "996733bf31598c7e2103225315f7c4664a3cb566b44bda843aa6db2ba2e00a48",
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
