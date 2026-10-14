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
    version: "0.2.4",
    release_url: "https://github.com/tinyhumansai/tinydocs/releases/tag/v0.2.4",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinydocs-module-0.2.4-ubuntu-24.04-x86_64.tar.gz",
            sha256: "2e9d3772a4434ea825914788f865476bc532fdc885097b35a95211202716fd9e",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinydocs-module-0.2.4-ubuntu-24.04-arm64.tar.gz",
            sha256: "9b63791a363a25f3f26d2bb155bb782b2ceb95c2f74ce6910fc650c2614c5a86",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinydocs-module-0.2.4-ubuntu-22.04-x86_64.tar.gz",
            sha256: "ac2f0aa79ae402103726b6f5adf23f90762071e23576e5dc6701293d49ad9427",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinydocs-module-0.2.4-ubuntu-22.04-arm64.tar.gz",
            sha256: "ff8bb22f796fcab37a0a4345f7e09b64cae99d81a5622f86cd940729f83994af",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinydocs-module-0.2.4-macos-26-arm64.tar.gz",
            sha256: "5b6105f050812c3331bdac8a52dbac21c734c5239b96642b4c2bd26dcd3fa727",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinydocs-module-0.2.4-macos-26-x86_64.tar.gz",
            sha256: "2b8eab7e244e720afa47525e4207e77cbbceb77cea416defe8be2a0c0abf661a",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinydocs-module-0.2.4-macos-15-arm64.tar.gz",
            sha256: "1038a30888e002b9c8cd1083be36b13fc99903a847060ad1b690eb0c80b6081b",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinydocs-module-0.2.4-macos-15-x86_64.tar.gz",
            sha256: "3ced33ab2ef2279848f1db10e6387cb557f3fab9f1221b25724c4fbc3709a9b2",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinydocs-module-0.2.4-windows-2025-x86_64.zip",
            sha256: "1b7fefa4a704f38dec8221182ca9f94f07e5dceaf8b66bd8acc72876fe39cb0f",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinydocs-module-0.2.4-windows-2022-x86_64.zip",
            sha256: "2145b031a68ac2b62daccfda2c723781073db38130e6dbed42055fb21dd23933",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinydocs-module-0.2.4-windows-11-arm64.zip",
            sha256: "2e2b91ea44f03cd026487cfbf94b7e3e3ea8525bb05b14d31bec56a8eb8ce4d9",
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
    version: "0.23.2",
    release_url: "https://github.com/tinyhumansai/tinywallet/releases/tag/v0.23.2",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinywallet-module-0.23.2-archlinux-arm64.tar.gz",
            sha256: "a859a611dd4ffc9dedf2c843d33dcacc61d240eea42ed3a434795b82b52c4d13",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinywallet-module-0.23.2-archlinux-x86_64.tar.gz",
            sha256: "9fb038d83887402267713146fc37f61b5f275cd819e445caf2329bf4eb208d3d",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinywallet-module-0.23.2-fedora-43-arm64.tar.gz",
            sha256: "7b575ebab88691c89130919339cc83a4757b64bfe5f235494e18aeb181aaab8e",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinywallet-module-0.23.2-fedora-43-x86_64.tar.gz",
            sha256: "b02a10325ba0f1208e6e06dfabd79d878f78377c4ade5fc60617edf9f3c78a88",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinywallet-module-0.23.2-fedora-44-arm64.tar.gz",
            sha256: "40979de5b1a08eee59a370bef3d32a815b6a51aef3dae2958c2f53e568a390cc",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinywallet-module-0.23.2-fedora-44-x86_64.tar.gz",
            sha256: "690deeececa612327eb34ec81b4521446c221c90190466c40a316a7aab9a7141",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinywallet-module-0.23.2-macos-15-arm64.tar.gz",
            sha256: "01bf9ab5be34862de7143c494354d72c06d5d81ad553fda36ce13f871d636469",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinywallet-module-0.23.2-macos-15-x86_64.tar.gz",
            sha256: "2d252e601f62d6ab76c61ed07c4282fa61316f51325fe8b45810199db26269f7",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinywallet-module-0.23.2-macos-26-arm64.tar.gz",
            sha256: "10a808ca83d6a10646146be738ebc958518116481d1599d7aaddc5c6fd468203",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinywallet-module-0.23.2-macos-26-x86_64.tar.gz",
            sha256: "a49d02b756579f9945dae02a3fd383ba8fcf3039f12e3fa97e2cfb98f9cdcb99",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinywallet-module-0.23.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "8da89114a625fefff1822e2b54abbbc65f85788f8bdeae17732cf0713c51eddf",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinywallet-module-0.23.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "8bf8f7597e07fb7453c2c96392333519ea3ef0dc402447f35d2910ed3c9e4a25",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinywallet-module-0.23.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "240aceaff4324312587d708c385806dbbd10e0fb71e782bd7c46a04e2606f481",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinywallet-module-0.23.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "96c82af6e6c2349b353789f33e617e7dba5bb63c4e50c4de49e3b359557c09f9",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinywallet-module-0.23.2-windows-11-arm64.zip",
            sha256: "edad022df52b317defabfc465fac0d57ff63c9c406797354e94e35a5f0744849",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinywallet-module-0.23.2-windows-2022-x86_64.zip",
            sha256: "1f44dce0e0bb5cac84ec9a1f596b86cda28e82d4198157f2f1faa0075e38ceb2",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinywallet-module-0.23.2-windows-2025-x86_64.zip",
            sha256: "d23eb02be466689258de016fdf84e1f8c971bb207abae6668168bb12f517f5bc",
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
