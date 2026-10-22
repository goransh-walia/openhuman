//! Registry record for the `tinyjuice` module.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

pub(crate) const TINYJUICE: ModuleRecord = ModuleRecord {
    id: "tinyjuice",
    description: "Content-aware tool-output compression and recoverable caching",
    bus_name: tinyjuice_bus::names::BUS_NAME,
    object_path: tinyjuice_bus::names::OBJECT_PATH,
    version: "0.11.2",
    release_url: "https://github.com/tinyhumansai/tinyjuice/releases/tag/v0.11.2",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinyjuice-module-0.11.2-archlinux-arm64.tar.gz",
            sha256: "356eb35a7b75435bc5f0eab6d25c25dff5b8ba578a66014a78f307e9f78c7758",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinyjuice-module-0.11.2-archlinux-x86_64.tar.gz",
            sha256: "459df33c5edadb33df273542e1438cf8d84777f5a25a727456069257b4bc11ed",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinyjuice-module-0.11.2-fedora-43-arm64.tar.gz",
            sha256: "fd3455a98dd771cfd5dc2c0dcbe24693b78a767877317e66a43a83766f01b188",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinyjuice-module-0.11.2-fedora-43-x86_64.tar.gz",
            sha256: "62134fc9b0ed536d356c27388f25232fc51f999fe536c0666ab67a21d2e1acc9",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinyjuice-module-0.11.2-fedora-44-arm64.tar.gz",
            sha256: "a60fc13016ef25e0436a911cfcb4078dd8e830b4eb61b2ea0b02f3698e26136e",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinyjuice-module-0.11.2-fedora-44-x86_64.tar.gz",
            sha256: "23e35f964bbd3aab66202ed304d0a1c77d35b66e715508aaec9daf98f1f3b12a",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyjuice-module-0.11.2-macos-15-arm64.tar.gz",
            sha256: "ea61ed6e8956d5fd0605bac9eba58e3f4b20934b7cbb31b31693480df3a0370d",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyjuice-module-0.11.2-macos-15-x86_64.tar.gz",
            sha256: "89aecb861a2cf613cd8350ba24bd0e2478f4347d54794056dbdd27bbe1646d9d",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyjuice-module-0.11.2-macos-26-arm64.tar.gz",
            sha256: "8c318acdcd90e7365fd49a297c7dc98c8512a55ef83547f5d4ad9574c421a3f1",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyjuice-module-0.11.2-macos-26-x86_64.tar.gz",
            sha256: "543c67a36fb523bd021db5a6ce01c16680a2c65ad0249db44eae8ea7fde654b1",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyjuice-module-0.11.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "61447cd9293a1c4608d6c570e8f9ef900258ae8b424a07627643677830616b77",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyjuice-module-0.11.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "ef63a0a98f086f68e4c965928bafa8c828cc622ea7d8cad698e818eaa20fefa6",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyjuice-module-0.11.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "a319a7f1fa2330c6428aeeb317698a9a661f9375bcd31d1c8c0fcd17f7ed66c1",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyjuice-module-0.11.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "045212c399169e43208d42a0f9d17aaffc661e8dbe657b9ddfb82ea29a5efc28",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyjuice-module-0.11.2-windows-11-arm64.zip",
            sha256: "44a1a62c6a08f9f3b6697c25e5625714a6ec027e02b70c245728a71cfc4ecc37",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyjuice-module-0.11.2-windows-2022-x86_64.zip",
            sha256: "930e4c466c55c6bb595e0ccc0d0bbb64e61413e3b4578671970749d8ed39930b",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyjuice-module-0.11.2-windows-2025-x86_64.zip",
            sha256: "610b05fdd7892555fbf62435ac2466a7be4a0daa5492247a98808e2972ddd6f2",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinyjuice",
        manifest: "crates/tinyjuice-module/Cargo.toml",
        package: "tinyjuice-module",
        lib_name: "tinyjuice_module",
        features: &[],
    },
};
