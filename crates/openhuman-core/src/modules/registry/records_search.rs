//! TinySearch web-search module. Published digests are added only from its
//! release manifest.
use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYSEARCH: ModuleRecord = ModuleRecord {
    id: "tinysearch",
    description:
        "Web search, grounded answers and page contents across providers through TinySearch",
    bus_name: tinysearch_bus::names::INTERFACE,
    object_path: tinysearch_bus::names::OBJECT_PATH,
    version: "0.5.0",
    release_url: "https://github.com/tinyhumansai/tinysearch/releases/tag/v0.5.0",
    // Verbatim from the published v0.5.0 checksum.toml; the same host set as
    // the other native modules (macOS, Ubuntu, Windows).
    assets: &[
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinysearch-0.5.0-macos-26-arm64.tar.gz",
            sha256: "a0e80c91e5e3dbad506ce501f56a0788902f7e40f4600b423e6359fe1877b5e9",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinysearch-0.5.0-macos-26-x86_64.tar.gz",
            sha256: "a173ced3c0741cd7a39b6742007d72ab129a3b9727e115b88e4ccc25c938af0b",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinysearch-0.5.0-macos-15-arm64.tar.gz",
            sha256: "e53492eabd52e930530fdc014eeb0749e0f57aa061a901d5e555b73d066dd9f0",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinysearch-0.5.0-macos-15-x86_64.tar.gz",
            sha256: "2ce7dd10773a07d7f0cac6d79fdc075d3ca6c147cc04abed6a2643f70f5cf8f9",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinysearch-0.5.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "e038b2529e40d834f4754f3566aaafc5cb5c4a2f0b643a389b4b66ba7f91db74",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinysearch-0.5.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "cf018f1330d7c0b6000ec876698ce1d4b6c2307710aba8df5bb38394f3f6522b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinysearch-0.5.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "4d14522a6ec6ee88c6de3681b1588904dbf6fd2565c1a5b762fca5b6fc04a5fe",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinysearch-0.5.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "0e610253f8183887c32f82ae3bb7f8e9808678c6034e984b94f06ba64900469e",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinysearch-0.5.0-windows-2025-x86_64.zip",
            sha256: "c7d6e2c1d993dbebdd64624c61d7a8888de42ea950c216e8e04399df2867c7d7",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinysearch-0.5.0-windows-2022-x86_64.zip",
            sha256: "042a73d4e50686ce61c6102a3e473401d811977116e98b1389ebd4293f8b968e",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinysearch-0.5.0-windows-11-arm64.zip",
            sha256: "be891330be69741e4783fd5812050890caf480d6df8a46d09ce5a9c65511433b",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinysearch",
        manifest: "crates/tinysearch/Cargo.toml",
        package: "tinysearch",
        lib_name: "tinysearch",
        features: &[],
    },
};
