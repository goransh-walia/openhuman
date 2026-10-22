//! Registry records for the `tinydocs` and `tinywallet` modules.

#[cfg(any(feature = "documents", feature = "web3"))]
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

#[cfg(feature = "documents")]
/// The `tinydocs` module: document synthesis, bounded extraction, image facts,
/// and PDF rendering.
///
/// Lazy, because a user who never asks for a document should not pay a download,
/// a `dlopen`, and the resident cost of a library that is never unloaded.
pub(crate) const TINYDOCS: ModuleRecord = ModuleRecord {
    id: "tinydocs",
    description: "Document synthesis, Office/PDF extraction, image inspection, and PDF rendering",
    bus_name: tinydocs_bus::names::BUS_NAME,
    object_path: tinydocs_bus::names::OBJECT_PATH,
    version: "0.2.5",
    release_url: "https://github.com/tinyhumansai/tinydocs/releases/tag/v0.2.5",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinydocs-module-0.2.5-ubuntu-24.04-x86_64.tar.gz",
            sha256: "7bef240f241cdc852871efa233454cd97febbbb7de6613c3de6c103e82b0fe62",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinydocs-module-0.2.5-ubuntu-24.04-arm64.tar.gz",
            sha256: "f61a645d4105ae5ece37d0f19f5494fa73be3df061bbb81e827cbed91cf53963",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinydocs-module-0.2.5-ubuntu-22.04-x86_64.tar.gz",
            sha256: "a942436faa90766a805f1b10ed9a4797471c552ba52869eeb8882331a6db5ad3",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinydocs-module-0.2.5-ubuntu-22.04-arm64.tar.gz",
            sha256: "b68565e8810c4cf7cedfef98f8421635b7b86828666ee1a799061404f6c43feb",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinydocs-module-0.2.5-macos-26-arm64.tar.gz",
            sha256: "d60ac5afe285857b687f1263f43716b27134ba309288a4b1849a7c6c0ac96760",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinydocs-module-0.2.5-macos-26-x86_64.tar.gz",
            sha256: "0feb2543a09a2f06a8b056bb8b728b28b0f2b86d85e1968e4113ff2a0d54475b",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinydocs-module-0.2.5-macos-15-arm64.tar.gz",
            sha256: "8352d968c5148980d945ea5db9e7229313528b15ffe9d7361a6d1ebbf251a7e4",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinydocs-module-0.2.5-macos-15-x86_64.tar.gz",
            sha256: "557023747a4b258a571776a653c71efba67099d4ae2bf4f52a521e150a3080c5",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinydocs-module-0.2.5-windows-2025-x86_64.zip",
            sha256: "a0b29c083a224a086ef4eacf64fbd6ac08565f1a71cdb7430976febf207ad69b",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinydocs-module-0.2.5-windows-2022-x86_64.zip",
            sha256: "bf8ec85f1f6255218452e009bb7b8c83be6557ec03f7721c6e08dd350d404ba2",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinydocs-module-0.2.5-windows-11-arm64.zip",
            sha256: "86007934b4bdc75a296116a5399027ea8af7efae25c7eac6bf35fbaf4e695f64",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinydocs",
        manifest: "crates/tinydocs-module/Cargo.toml",
        package: "tinydocs-module",
        lib_name: "tinydocs_module",
        features: &[],
    },
};

#[cfg(feature = "web3")]
/// The `tinywallet` module: native wallet, Web3 quote, and x402 operations.
///
/// Lazy for the same reason as [`TINYDOCS`], and more so: most sessions never
/// touch a wallet, and this artifact carries `bitcoin` and a native `secp256k1`
/// build that would otherwise be resident for all of them.
///
/// **This host sends it the recovery phrase, over confidential calls, and never
/// derives or signs itself.** All four chains — Bitcoin, EVM, Solana and Tron —
/// derive and sign inside the module. Web3 quotes and x402 payments, budgets,
/// and ledger state also live there. The host supplies credential custody,
/// authenticated transports, and exact-facts approval callbacks. This binary
/// depends on `tinywallet-bus` for the wire contract rather than implementation
/// crates.
///
/// The phrase is only sent to a module tinybus has attested *and* whose digest
/// matches one of the entries below — `super::wallet::attested_proxy` checks
/// this table itself rather than trusting that some check happened.
///
/// The contract also exposes `ExportKey` for downstream hosts that must drive
/// a signer locally; OpenHuman itself does not call it.
///
/// The pinned v0.20.0 release (contract 1.14) carries the native wallet, Web3,
/// and x402 bus operations, including mnemonic word-count validation. Secret-bearing
/// operations remain confidential and require both attestation and a matching
/// pinned digest.
pub(crate) const TINYWALLET: ModuleRecord = ModuleRecord {
    id: "tinywallet",
    description: "Transaction building and assembly for Bitcoin, EVM, Solana and Tron",
    bus_name: tinywallet_bus::names::BUS_NAME,
    object_path: tinywallet_bus::names::OBJECT_PATH,
    version: "0.24.1",
    release_url: "https://github.com/tinyhumansai/tinywallet/releases/tag/v0.24.1",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinywallet-module-0.24.1-archlinux-arm64.tar.gz",
            sha256: "f0e013d305d35744911e4e88ae391bd5b698ca9b5d4b5965ff2cf7e038022a55",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinywallet-module-0.24.1-archlinux-x86_64.tar.gz",
            sha256: "e8feeacc13150230db39885471868f3cb983d32cdca3b54076cfa3b7b99ca291",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinywallet-module-0.24.1-fedora-43-arm64.tar.gz",
            sha256: "a5eda11c12b04838101113bb691809cccaab4f02b63927ed87b9ae23354e8024",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinywallet-module-0.24.1-fedora-43-x86_64.tar.gz",
            sha256: "81806447a07fb1e5f2e2e11c44cc3bb673b9583836b468086abda33cac7d74c6",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinywallet-module-0.24.1-fedora-44-arm64.tar.gz",
            sha256: "1b46b6df147080672ed2f87c9b560f3c4ab1d5edabe16f98e66e3cc1df0556fc",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinywallet-module-0.24.1-fedora-44-x86_64.tar.gz",
            sha256: "021973415b5e6ebde7f903e9ecd7727a3511fe22bf5f4452720eb6d6b8898a7f",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinywallet-module-0.24.1-macos-15-arm64.tar.gz",
            sha256: "018451ecd682ca8cdd45667d9d7a10e8200a666f1ce5e76c7784f1ed1fe2a59b",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinywallet-module-0.24.1-macos-15-x86_64.tar.gz",
            sha256: "8c9887fe1a321e0f13b1af3f9129e6db02186fd791d6769e3a80bae964ddb240",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinywallet-module-0.24.1-macos-26-arm64.tar.gz",
            sha256: "d0896d7fbb8936542cdfc4c0d138a262982d599ec015401a6dbd1d4410a39d3a",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinywallet-module-0.24.1-macos-26-x86_64.tar.gz",
            sha256: "19409a538aa0fd04f246ebcbaacd4aefc7c55279d49f3d066a44257432617314",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinywallet-module-0.24.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "0b7da4811e31e2d7239c62fdb30c8ac4c45667b596b27033f4f3915655fccbe7",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinywallet-module-0.24.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "5b1e62a9fa88939cf1711dc0480a6298d240ce2e49cbd4a448a2f967a2f30255",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinywallet-module-0.24.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "e50ffef6a61dfbfffc0be0ad32e8388ee72c8803f63b71f435ddc6c47c337650",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinywallet-module-0.24.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "11f66fa19923f1fd151252294b947becf6eb5edafe6a7750681daf74c028d28c",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinywallet-module-0.24.1-windows-11-arm64.zip",
            sha256: "ad8f432919fa78ca7a0828b8c3ad02fcd1b65855cbaf1f76355dcd0423ff7415",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinywallet-module-0.24.1-windows-2022-x86_64.zip",
            sha256: "03badd28042ff3a211a489bafb42ac1fa90bb320de66bb73ef5317c7b4c560d3",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinywallet-module-0.24.1-windows-2025-x86_64.zip",
            sha256: "53fc1c72e207769fa471373320b0ca73c067c2f6b32a65e73f8fc7ef18a0fa80",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinywallet",
        manifest: "crates/tinywallet-module/Cargo.toml",
        package: "tinywallet-module",
        lib_name: "tinywallet_module",
        features: &[],
    },
};
