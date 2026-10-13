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
    version: "0.18.0",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.18.0",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-rolling-arm64",
            archive: "tinymcp-0.18.0-archlinux-arm64.tar.gz",
            sha256: "2926214fa8ea01a509f21264e9b39f6647bf9efd48655aa2b5e04f010eec4765",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinymcp-0.18.0-archlinux-x86_64.tar.gz",
            sha256: "a2e3273a098325c935c6d64382a9a96bd96f2a0e557705d292e60dcd60886732",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinymcp-0.18.0-fedora-43-arm64.tar.gz",
            sha256: "ab0345d9935546905181e2b29d4cbf1033c52c9ba5f027aa7a68b58d5fa64966",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinymcp-0.18.0-fedora-43-x86_64.tar.gz",
            sha256: "7618fb26992886ee53d94a8e67de725e2cc02b65cd4436989c5f76b7ed44c097",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinymcp-0.18.0-fedora-44-arm64.tar.gz",
            sha256: "9502ef005fe2fff50d5c931f96e0fe496f04f8aaa5b7c42d845159c2ecd1d657",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinymcp-0.18.0-fedora-44-x86_64.tar.gz",
            sha256: "d73573e2a6481e9863db4ca4cac5c422c1dca8999fca99aca97f645471548865",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.18.0-macos-15-arm64.tar.gz",
            sha256: "f2704ff75b1399d2cc92d965d0c5cb9085b8e1f778b785f6ed6c9a184ea64f95",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.18.0-macos-15-x86_64.tar.gz",
            sha256: "e7036da10f52af9248478e507897414b304c75b28e9156013124790ac4f97d23",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.18.0-macos-26-arm64.tar.gz",
            sha256: "e49a2d111988787e517694fb1b678e890da24cb77f4554bdce3d7ce973ea2d17",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.18.0-macos-26-x86_64.tar.gz",
            sha256: "4fc5495b98ec119c0f939cafeeac8c393a0fe297dac5b384a21d7a7d82bb3800",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.18.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "47c3934ca545dec412351fbd06f972c3d8ee9f9f87cec102d4073315b11aa5d2",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.18.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "259dd599a831a7e25c32687f9e1c8b9e7ebba0aec2bb34356921a4a35520ae98",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.18.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "7fa57dd1f7b3d7585814c70e8b3dc65e79183fcc8cbda140ad68de80f858125a",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.18.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "3793da284a0698506c456d0117e0b3f774707c10ad53b7af65c2ab2d813153b4",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.18.0-windows-11-arm64.zip",
            sha256: "0479b54dd843ef0d1c0d048de76f10f5f09d0298247227ead83b41859d7828bd",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.18.0-windows-2022-x86_64.zip",
            sha256: "c16ea238bb7da168e0c96ddde69305e41df27a25a25073f7df4740d9b96a4f6d",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.18.0-windows-2025-x86_64.zip",
            sha256: "8899c643de2cd9274b06efc8dccbd13379bf7bbd570e87b276a315c0f1f84695",
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
    version: "0.18.0",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.18.0",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinyconnectors-0.18.0-archlinux-arm64.tar.gz",
            sha256: "1f88b89aab79f443a89e894d3b767d46a75e60aae44270b8a70fb64a940b15f9",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinyconnectors-0.18.0-archlinux-x86_64.tar.gz",
            sha256: "9eb635f95d4d483dbfe77605c73cf2762b141aeb7643b648b28810424b01c0e2",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinyconnectors-0.18.0-fedora-43-arm64.tar.gz",
            sha256: "b70cf6b82106bfd3de16d126a32ec9bda5c1562fd30910ec3c41b5feedd44f40",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinyconnectors-0.18.0-fedora-43-x86_64.tar.gz",
            sha256: "b85bdba6e3a323f57a8f15e111256a3c4ba0e3964f53d3ed5f487c50a4fc3de5",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinyconnectors-0.18.0-fedora-44-arm64.tar.gz",
            sha256: "773af57110b4a398b7d7c23491893dc9233fe52f517ba308435cf8235fd56c3d",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinyconnectors-0.18.0-fedora-44-x86_64.tar.gz",
            sha256: "8b1647e1a7207ad96044bb741e704dc8f56ec578ce30c862eb13d3af4b7c9bd3",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.18.0-macos-15-arm64.tar.gz",
            sha256: "d7914003e3508f202701a7469950f631347a9bdcc3791c41666e40aa1e5f9249",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.18.0-macos-15-x86_64.tar.gz",
            sha256: "7df3d800fd42d99c873d0ad915e14382f1cade7bb3c3e0f6fa6f9363f645adc3",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.18.0-macos-26-arm64.tar.gz",
            sha256: "a477e127a4aaa3143dee1bb6edf267af9d25fa60e3aa2c24793ad6e2a2d92ef3",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.18.0-macos-26-x86_64.tar.gz",
            sha256: "7ef87939be10f1e7b002d509442043378ed8ee53eebf0a34e95813dc79bab157",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.18.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "4828242bbdc8026566eff76f8748bf30c5bc65cd668967a2b649c93dc4968a6c",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.18.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "541e749cac08df822ffe95b604ad9a95d9640a6a1290fda1b6218196baf0a571",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.18.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "1699dec831e5876d9aad878dce155a4c30c3f872c58f50b14188973e16dde4e8",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.18.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "88a3298c62fd20418ce5e563e7261d322a8b2a55b3b9e6f85952c70c77c4d038",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.18.0-windows-11-arm64.zip",
            sha256: "c6a70997bb42f72b24009006d3b00d85d674ef619447dea58c5d17a9ab6b6c11",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.18.0-windows-2022-x86_64.zip",
            sha256: "5a35fd8f5e8733e26089ad5ee7cb10294a4c8125ebfa9f823f072a18cf92be89",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.18.0-windows-2025-x86_64.zip",
            sha256: "2d33cffaded0125f7d3cbe2bea84eb9680c965229b0d6dea28277b0e71e9a35f",
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
