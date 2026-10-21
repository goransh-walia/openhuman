//! Registry record for the `tinyvoice` module.

#[cfg(feature = "voice")]
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

#[cfg(feature = "voice")]
/// The `tinyvoice` module: the host-agnostic half of the voice pipeline.
///
/// STT hallucination detection used by the voice transcription pipeline.
///
/// Lazy: transcript screening loads the module only when voice transcription
/// needs it.
pub(crate) const TINYVOICE: ModuleRecord = ModuleRecord {
    id: "tinyvoice",
    description: "Voice transcript hallucination detection",
    bus_name: tinyvoice_bus::names::BUS_NAME,
    object_path: tinyvoice_bus::names::OBJECT_PATH,
    version: "0.3.4",
    release_url: "https://github.com/tinyhumansai/tinyvoice/releases/tag/v0.3.4",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyvoice-module-0.3.4-ubuntu-24.04-x86_64.tar.gz",
            sha256: "ba5417d7442d3cc8c800f89f0e2e765c4f30c30bf8257a8030022bc2416ca175",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyvoice-module-0.3.4-ubuntu-24.04-arm64.tar.gz",
            sha256: "1ed67f4f36538e5ad5052d796a5c2bc647b0edd2cf78366c8d9be521e4e516b7",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyvoice-module-0.3.4-ubuntu-22.04-x86_64.tar.gz",
            sha256: "35e5529cd3e6ab3d6ca062bc110181526851ab13d59c34d8a3febd93d21262fa",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyvoice-module-0.3.4-ubuntu-22.04-arm64.tar.gz",
            sha256: "bc31815f65fff0e44b47f0da8eac10a4a404fb516d0c093649ed4e1de005e925",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyvoice-module-0.3.4-macos-26-arm64.tar.gz",
            sha256: "c9c729ba646a5069b30e4b84cdeb53f3ec274406f10c10955411862eacc246f1",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyvoice-module-0.3.4-macos-26-x86_64.tar.gz",
            sha256: "5ad4ca051265063338510d891d894c347d76356828605f8c5f793986acddfdda",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyvoice-module-0.3.4-macos-15-arm64.tar.gz",
            sha256: "fa2c380ce034e31f96f3faae4e7d7651e9f2df5614977cae7c5bf0339dd2a8ab",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyvoice-module-0.3.4-macos-15-x86_64.tar.gz",
            sha256: "29ec1e0097a6da61e2da5c6f6d2290ddb74217a72840fd8e22d51cacdd288f6f",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyvoice-module-0.3.4-windows-2025-x86_64.zip",
            sha256: "a4acbbd05d8965de326e9f37241c28fa14cef696062b013720c96adbd56eb4be",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyvoice-module-0.3.4-windows-2022-x86_64.zip",
            sha256: "b0aff0e3c11fc825078fa62a4ead24cec41eb938ac6d45f6231e468e6bb1283a",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyvoice-module-0.3.4-windows-11-arm64.zip",
            sha256: "c9bb31d5bbaa0d614262a92ceaa793755bb651992f6b18c768062154b42dbebd",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinyvoice",
        manifest: "crates/tinyvoice-module/Cargo.toml",
        package: "tinyvoice-module",
        lib_name: "tinyvoice_module",
        features: &[],
    },
};
