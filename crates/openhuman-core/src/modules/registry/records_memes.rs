//! Registry record for the `tinymemes` module.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

/// The `tinymemes` module: final-reply slang and meme remixing, loaded on
/// first use. The host serves the callbacks it asks for (inference, Jev,
/// search, index storage, delivery).
pub(crate) const TINYMEMES: ModuleRecord = ModuleRecord {
    id: "tinymemes",
    description: "Final-reply slang and meme remixing through TinyMemes",
    bus_name: tinymemes_bus::STATE_INTERFACE,
    object_path: tinymemes_bus::STATE_PATH,
    version: "0.3.1",
    release_url: "https://github.com/tinyhumansai/tinymemes/releases/tag/v0.3.1",
    assets: &[
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemes-module-0.3.1-macos-15-arm64.tar.gz",
            sha256: "2779bc03a4aea58c92619c64a8f9addb7da2329f9e45b65b8c1546d0bbc92c06",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemes-module-0.3.1-macos-15-x86_64.tar.gz",
            sha256: "67d883bd3db47492e6b45d03018aa3cd681b52d3e1a3f74376e9ab13257959e6",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemes-module-0.3.1-macos-26-arm64.tar.gz",
            sha256: "1cb0add4ed4209736364b9291ba9d28d8cfc66e8f2f76310578ef18a5a541cc0",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemes-module-0.3.1-macos-26-x86_64.tar.gz",
            sha256: "6919a98c0f9d3b3c2aaee2aa17f64b8142f175599d051f82a01e0feb67280180",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemes-module-0.3.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "976a06647e59b448ed66bffdaaf0f26032d3d3f458ac8099ccbb53e822da24b7",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemes-module-0.3.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "06d9f095f5e882638930c4cc092e6e7f8426d8ee23ebc01ffa8762b3ea0827bf",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemes-module-0.3.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "56e804d498637fe2e3352c997c43164be659c1f966b80e549984f404c6440804",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemes-module-0.3.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "0f8f0e36a311554c248ef5da686bc7f96c2dd5e013348bd85f4ce2336e83157e",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemes-module-0.3.1-windows-11-arm64.zip",
            sha256: "4101ddc2ffba7714bf2ee68e93219d2a3043a83e37741715fd1aaa1f0ba6771a",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemes-module-0.3.1-windows-2022-x86_64.zip",
            sha256: "f222cacadb4940753d14054f6135c3f4e81216c0649c36c2e6bfde365736e321",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemes-module-0.3.1-windows-2025-x86_64.zip",
            sha256: "4eeff7a6fada6aa01f82c5a54ffdccbfa2717b220296a8650e3cf3e06bddb03f",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinymemes",
        manifest: "crates/tinymemes-module/Cargo.toml",
        package: "tinymemes-module",
        lib_name: "tinymemes_module",
        features: &[],
    },
};
