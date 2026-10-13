//! Registry record for the `tinyvoice` module.

#[cfg(feature = "voice")]
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

#[cfg(feature = "voice")]
/// The `tinyvoice` module: the host-agnostic half of the voice pipeline.
///
/// Wake-word gating, fast-path command routing, STT hallucination detection,
/// and the capture-side audio work (downmix, resample, silence gate, WAV
/// framing).
///
/// Lazy, and more clearly so than the others: voice is opt-in twice over — a
/// user has to enable dictation or always-on listening before any of this runs
/// — so a session that never speaks should not pay a download or a `dlopen`.
///
/// Devices and audio processing live in the compiled module. Hosts poll bounded
/// capture batches and apply product permission, privacy and transcription policy.
pub(crate) const TINYVOICE: ModuleRecord = ModuleRecord {
    id: "tinyvoice",
    description: "Wake-word gating, command routing, hallucination detection, capture audio",
    bus_name: tinyvoice_bus::names::BUS_NAME,
    object_path: tinyvoice_bus::names::OBJECT_PATH,
    version: "0.3.0",
    release_url: "https://github.com/tinyhumansai/tinyvoice/releases/tag/v0.3.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyvoice-module-0.3.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "945fdd762be37c779bdf16b96da03109ce82c7b43d5bb9865e0dfc32e239d883",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyvoice-module-0.3.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "fadbe5b4b9d47744a21096838c10e2ad03b3be43f690f8cef7deb1d2fc1bc427",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyvoice-module-0.3.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "5e1f4b18a6662244fc1488166db153f9df60c651828e44e8bcfd55dc46698087",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyvoice-module-0.3.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "d20f000a9343167016fbfef7007ee6fdd8b84f022cdc15d96c203a0b43aab058",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyvoice-module-0.3.0-macos-26-arm64.tar.gz",
            sha256: "4cdbab216808163d8fe33bf9c55eea1355fe1d84cd265167971dac2763f65807",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyvoice-module-0.3.0-macos-26-x86_64.tar.gz",
            sha256: "8f12b00fa3474cd5ba2baef3e0e8f8dfb30a401f103bf70d339d8d997ce79597",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyvoice-module-0.3.0-macos-15-arm64.tar.gz",
            sha256: "91b231f6b6e48e3f78a9ae8b3fecab606c7d20f004fe300c030b883df845fd82",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyvoice-module-0.3.0-macos-15-x86_64.tar.gz",
            sha256: "f9f4ed3fa6e3615c7f747a619b8c32a2ee252d51541226e27142422273104dad",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyvoice-module-0.3.0-windows-2025-x86_64.zip",
            sha256: "ff91872dee9ac3ab098bea1830fe1bebd6f16a8cc3267e02ee36a74d0bc48469",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyvoice-module-0.3.0-windows-2022-x86_64.zip",
            sha256: "b2485d846a3287c47da303237680e0cbe76a210e5ef2f25027220cde4ffb8ed4",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyvoice-module-0.3.0-windows-11-arm64.zip",
            sha256: "7b7e247fb2b6f8be6948ffbe21ed9244e01dac6f367db12b5a78d014d13b8ae9",
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
