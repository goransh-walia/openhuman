//! Registry record for the `tinyvoice` module.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

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
/// **The VAD deliberately does not come through here.** A segmenter is driven
/// Continuous capture and its device callback are module-owned and read in
/// bounded batches. The VAD remains in the host because it is a local
/// per-frame policy loop; captured samples cross the bus in bounded batches,
/// never through a call made from the audio callback. See [`super::voice`].
pub(crate) const TINYVOICE: ModuleRecord = ModuleRecord {
    id: "tinyvoice",
    description: "Wake-word gating, command routing, hallucination detection, capture audio",
    bus_name: "ai.tinyhumans.tinyvoice.Voice",
    object_path: "/ai/tinyhumans/tinyvoice/Voice",
    version: "0.1.14",
    release_url: "https://github.com/tinyhumansai/tinyvoice/releases/tag/v0.1.14",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyvoice-module-0.1.14-ubuntu-24.04-x86_64.tar.gz",
            sha256: "682b56ec246e021708a71a587a9fa55625c3964b10676f9d1220ee4eccdd55a1",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyvoice-module-0.1.14-ubuntu-24.04-arm64.tar.gz",
            sha256: "87cde6de6da8c9d9f392fe43a8547be861363596f64a326cf9deb985e260762d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyvoice-module-0.1.14-ubuntu-22.04-x86_64.tar.gz",
            sha256: "9bab7b7d6d5e36526f13a02209cd3454ea53ecf8ba68894ffa5f2376765cab71",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyvoice-module-0.1.14-ubuntu-22.04-arm64.tar.gz",
            sha256: "2c77ed20a9745086a1d8e481a7719accaff82464b546671b0070b78610e77628",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyvoice-module-0.1.14-macos-26-arm64.tar.gz",
            sha256: "f2856e78eda2f53cacc4799c2a068a5626779f01bff60a792c1f7e37c56324cb",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyvoice-module-0.1.14-macos-26-x86_64.tar.gz",
            sha256: "e4ff4337bcb4fa60c3549e83fe573060d8669b744996fa75d321542b8e6841f3",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyvoice-module-0.1.14-macos-15-arm64.tar.gz",
            sha256: "c3e6a29cd39617d91463d4315e1dfe432a12f0579116173efac4406a9e2d65a7",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyvoice-module-0.1.14-macos-15-x86_64.tar.gz",
            sha256: "86d9e654391cf09dc40ac27e0128d614b03ff3853e9bb010b9b31d914e0b1f23",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyvoice-module-0.1.14-windows-2025-x86_64.zip",
            sha256: "f6a636891710c7443306fdcf6067f24b72b1b98a9873596669377540da0b037b",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyvoice-module-0.1.14-windows-2022-x86_64.zip",
            sha256: "e5b538e69dfc0cec83adcb6240f09d6d0cb126df76621f8eb63b5da901119baa",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyvoice-module-0.1.14-windows-11-arm64.zip",
            sha256: "8452b6099a0a1debe412e3e69f8591756c5ea49f9e5a112f138fe2e590a0a418",
        },
    ],
    load: LoadPolicy::Lazy,
};
