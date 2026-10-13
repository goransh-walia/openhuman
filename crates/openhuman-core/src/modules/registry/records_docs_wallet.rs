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
    version: "0.2.3",
    release_url: "https://github.com/tinyhumansai/tinydocs/releases/tag/v0.2.3",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinydocs-module-0.2.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "2729bc156d81a68881b9b5bc0fc1947f8e2c59c49cc057c16c9f5822b6dc2eba",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinydocs-module-0.2.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "61e8448762f1a0ce59d023db9108be883c9e0ec7d66d8b37b517d71d9b30a8cf",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinydocs-module-0.2.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "bf8067830b235130e4f2ae7b935102818b679994422a4a692162b6722ebcef37",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinydocs-module-0.2.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "a35a0bf4e8f48490f798d257570ce5fecfe2c4e01ddc504e904e1ff960b372d4",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinydocs-module-0.2.3-macos-26-arm64.tar.gz",
            sha256: "04eff90653bb3edb8497064ccc9b09ec5bdae7292f693af57cec1a333a3ee42d",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinydocs-module-0.2.3-macos-26-x86_64.tar.gz",
            sha256: "604f687a8ceddad9daceba315a65904d46a5fc886b0bbf402d968e1bef42f8d5",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinydocs-module-0.2.3-macos-15-arm64.tar.gz",
            sha256: "d7c44ef66966878427772b66b3c435d3bb24f07924bb817cfacc63f509cd4284",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinydocs-module-0.2.3-macos-15-x86_64.tar.gz",
            sha256: "8d5b9c83bdc8147ca78cd0f640e50a9cc3672a27dffaf0a901664d157a7db9f8",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinydocs-module-0.2.3-windows-2025-x86_64.zip",
            sha256: "137951a08337e5e5848ec3013234b0959d9d20bfd8842b74d1571f17f864222e",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinydocs-module-0.2.3-windows-2022-x86_64.zip",
            sha256: "52aed32ea1f52637e62ff85eb6a474b03086ef356dd653e123f5a09b02b05742",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinydocs-module-0.2.3-windows-11-arm64.zip",
            sha256: "ee98efdc442a3b54c899fdb6e5d16960c308c3522e0655413dd87cee1f3d780e",
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
    version: "0.22.0",
    release_url: "https://github.com/tinyhumansai/tinywallet/releases/tag/v0.22.0",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinywallet-module-0.22.0-archlinux-arm64.tar.gz",
            sha256: "c0a7911452b1bf1461c945d45170305122dcb791c0ea3378411b1332cc4e7013",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinywallet-module-0.22.0-archlinux-x86_64.tar.gz",
            sha256: "f15aa217405a41a7d4b53462e2f9f3e3745184ec1de9e54f79deca54a4162177",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinywallet-module-0.22.0-fedora-43-arm64.tar.gz",
            sha256: "c9a92837299aa2a43883341390ecceb54e2e8567c23fe9963be51077d9726c1e",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinywallet-module-0.22.0-fedora-43-x86_64.tar.gz",
            sha256: "a19486e72204f0150466e06372538f70b761ddd7ff49fab5bd736f7bc48581a4",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinywallet-module-0.22.0-fedora-44-arm64.tar.gz",
            sha256: "5ac8b865224d229d3d7bd199cd651e3862d4658fd125aae4121a0a0e709edde2",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinywallet-module-0.22.0-fedora-44-x86_64.tar.gz",
            sha256: "d404e7e484d09835c843731ddbbc5a93354071d8ed900792e42f73e56c4ba13f",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinywallet-module-0.22.0-macos-15-arm64.tar.gz",
            sha256: "1a173add7bd67565583f0cd557584dd79b2d64b9330aca93a5ded27446fc24c7",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinywallet-module-0.22.0-macos-15-x86_64.tar.gz",
            sha256: "99a6d43a5f9668b36c5d98ec51daf3c343515af870dbf4745abdc6cf4ef2cfe7",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinywallet-module-0.22.0-macos-26-arm64.tar.gz",
            sha256: "4da19b9a12677db4187ef2076a44341066bf96976dbe89845fff32ff7990b0dd",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinywallet-module-0.22.0-macos-26-x86_64.tar.gz",
            sha256: "a00ec245f3b72810e7c25d513bb5976dc9ada07265f8bdec2217d217049c4399",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinywallet-module-0.22.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "c37c0c477e89b38013cbe6e8d8f62ddff3cf08c94e0452f441ae041265f3e7ab",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinywallet-module-0.22.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "df6c0b4ad6b560242186357f10d29f490ac278fb2f94c6f80934c8723f64130d",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinywallet-module-0.22.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "c31bf29e7f6004be268e484651621a8e2c9efc3221a3d91285318888079dbb41",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinywallet-module-0.22.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "d0ef3d6833c972c18a917f965b71e7df61087c6eb61496ff6af9ca76605325e2",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinywallet-module-0.22.0-windows-11-arm64.zip",
            sha256: "02969de77cd63c2d889dc65231de6327fa39b4b39ffbacee2c4524dc34f44512",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinywallet-module-0.22.0-windows-2022-x86_64.zip",
            sha256: "401cc74695b52a40806713ece3f31c971d23ee3cd0529b231aa36a809cb015a0",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinywallet-module-0.22.0-windows-2025-x86_64.zip",
            sha256: "880c52be4634c0693b64769d58462605075954430b36a7dc7fff7dc49ecdf4fe",
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
