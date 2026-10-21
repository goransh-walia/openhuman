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
    version: "0.19.5",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.19.5",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-rolling-arm64",
            archive: "tinymcp-0.19.5-archlinux-arm64.tar.gz",
            sha256: "ca2e8bc52523a9e719ec4ac14e5be52ba9389afb57faeaaeb9cbd56000b29528",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinymcp-0.19.5-archlinux-x86_64.tar.gz",
            sha256: "33f4aab3a961faeb3fc64ec8055b712d73dc297152eb8002c432a1855d1869c3",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinymcp-0.19.5-fedora-43-arm64.tar.gz",
            sha256: "fa66fa9092c176e6a37beeb2af157d5acf530b7bf3c7786459e09daf2f8a75ba",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinymcp-0.19.5-fedora-43-x86_64.tar.gz",
            sha256: "09965afec03286005bae27e7dd7c29251645f42bdc933b3457190d655e87960c",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinymcp-0.19.5-fedora-44-arm64.tar.gz",
            sha256: "62be25f9927ac08c03a7638386e0c904e7313d4255a3b85264774165bb3feb6d",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinymcp-0.19.5-fedora-44-x86_64.tar.gz",
            sha256: "7e4fd1a3261cbc166ed358cafb93a665d2464197beb2f896d44f0382f07ec2c6",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.19.5-macos-15-arm64.tar.gz",
            sha256: "1e2023a2b6526a9a465916c18d714306dd953d5c887aa0226a2fffd15a77dab0",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.19.5-macos-15-x86_64.tar.gz",
            sha256: "e3147cb5ab4599f3c308807137d902717677c59f15954fa48b73201a1a3aed15",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.19.5-macos-26-arm64.tar.gz",
            sha256: "0c2df4171521bb80122ad9f3eca01dab34def059023f69f97eac2337cadb2a16",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.19.5-macos-26-x86_64.tar.gz",
            sha256: "d07df68db79ce77afe03ae387e9304b59fb22bdebc95ef7bf3447326f8b1f3c5",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.19.5-ubuntu-22.04-arm64.tar.gz",
            sha256: "c80340656ca14cc75802d8ee896a725a83b589eaba7c967afafb255b731dc785",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.19.5-ubuntu-22.04-x86_64.tar.gz",
            sha256: "db59bec7a6e81195b15ac2ab23819da2515ffe0d610896de6a3732cf9111f74e",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.19.5-ubuntu-24.04-arm64.tar.gz",
            sha256: "1706005201deb02965c8d5c3abe96bb85bf58c7fb301ecc64ca6541a634568f2",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.19.5-ubuntu-24.04-x86_64.tar.gz",
            sha256: "de0caac98c9059fe36338d27eb18d1c11110f68c13e8cd524dd6c5f72efe0d51",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.19.5-windows-11-arm64.zip",
            sha256: "20c059fda61b1f32fb0d957b55b0994708b0f8fb8f604cfe15b652e6ecdb84c3",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.19.5-windows-2022-x86_64.zip",
            sha256: "91db87594204b69032184243799fe858439e90035000bad9420b1629b5fe9aa8",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.19.5-windows-2025-x86_64.zip",
            sha256: "9898dd468f31ec3d11b14d69b7c3cda1432ba726fcb608fe50da5a3045719916",
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
