use super::{find, ALL};
use tinybus::module::platform::candidates_for;

#[cfg(feature = "security-module")]
#[test]
fn tinysecurity_is_eager_and_matches_the_typed_policy_contract() {
    let record = find("tinysecurity").expect("compiled native security module");
    assert_eq!(record.bus_name, tinysecurity_bus::names::INTERFACE);
    assert_eq!(record.object_path, tinysecurity_bus::names::OBJECT_PATH);
    assert_eq!(record.load, crate::modules::LoadPolicy::Eager);
}

#[test]
fn tinycomputer_registry_matches_bus_contract_and_published_release() {
    let desktop = find("tinycomputer").expect("compiled computer module");
    assert_eq!(desktop.bus_name, tinycomputer_bus::names::INTERFACE);
    assert_eq!(desktop.object_path, tinycomputer_bus::names::OBJECT_PATH);
    assert_eq!(desktop.version, "0.11.0");
    assert_eq!(desktop.assets.len(), 16);
    assert_eq!(tinycomputer_bus::CONTRACT_VERSION, (2, 12));
    assert_eq!(
        desktop.asset_for("ubuntu-24.04-x86_64").unwrap().sha256,
        "430426bdbc6f8ad8cc5ed17d6f3bd05f73b4d4fe747a8fc9b77f0b3f4fd3506b"
    );
    assert_eq!(
        desktop.asset_for("macos-26-arm64").unwrap().sha256,
        "3a8e20a809262be2154c1b7339fed2a627c13c32b2a7b9de52e4d54ced94a994"
    );
}

#[test]
#[cfg(feature = "web3")]
fn tinywallet_registry_matches_the_verified_v019_release_manifest() {
    let wallet = find("tinywallet").expect("compiled TinyWallet module");
    assert_eq!(wallet.version, "0.19.0");
    #[cfg(feature = "web3")]
    assert_eq!(tinywallet_bus::CONTRACT_VERSION, (1, 13));
    assert_eq!(
        wallet.release_url,
        "https://github.com/tinyhumansai/tinywallet/releases/tag/v0.19.0"
    );

    let mut actual = wallet
        .assets
        .iter()
        .map(|asset| (asset.host_key, asset.archive, asset.sha256))
        .collect::<Vec<_>>();
    assert_eq!(wallet.assets.len(), 17);
    let mut published = [
        (
            "archlinux-arm64",
            "tinywallet-module-0.19.0-archlinux-arm64.tar.gz",
            "59863d52205fabe486a16dfbf573a007d19c3237236cee3a8ff236281cdf6ba9",
        ),
        (
            "archlinux-x86_64",
            "tinywallet-module-0.19.0-archlinux-x86_64.tar.gz",
            "645679737effe617f566af7349dc7153f29dac21fa7a377a610ccf72054e7454",
        ),
        (
            "fedora-43-arm64",
            "tinywallet-module-0.19.0-fedora-43-arm64.tar.gz",
            "c05ff2da4101246b7a50a37a44632a8502a5747f5e7cc416258fb728782d91b6",
        ),
        (
            "fedora-43-x86_64",
            "tinywallet-module-0.19.0-fedora-43-x86_64.tar.gz",
            "c4399bbb1c9d3c42545fb78474f548337cfc4a96ebb49ce50912ca4c85dd3aa4",
        ),
        (
            "fedora-44-arm64",
            "tinywallet-module-0.19.0-fedora-44-arm64.tar.gz",
            "d17075dd175d0bd3bebe31350a3bcc58320c698f4c54f69691f85dd54297fdb9",
        ),
        (
            "fedora-44-x86_64",
            "tinywallet-module-0.19.0-fedora-44-x86_64.tar.gz",
            "ef64e56e536a2661ae0ddcd799632819c02f391a5e7f98431adc184f7df45adb",
        ),
        (
            "macos-15-arm64",
            "tinywallet-module-0.19.0-macos-15-arm64.tar.gz",
            "15397175e39dffbfe372bb40a4dccd6e03440ed5925043d41df058a335e1427d",
        ),
        (
            "macos-15-x86_64",
            "tinywallet-module-0.19.0-macos-15-x86_64.tar.gz",
            "389e68114a4d1243afe04e8143e7f6ebd321c1cc24b54be2d768c7ee8643c7b1",
        ),
        (
            "macos-26-arm64",
            "tinywallet-module-0.19.0-macos-26-arm64.tar.gz",
            "bff93e7c33fcf663c0b69117add0e249c461233a58eb5ba2f1b753a9a8944350",
        ),
        (
            "macos-26-x86_64",
            "tinywallet-module-0.19.0-macos-26-x86_64.tar.gz",
            "72e1ef5c1047c0ac848abb9cd1cf69e6639ac4e8527496979406922852a4d2d3",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinywallet-module-0.19.0-ubuntu-22.04-arm64.tar.gz",
            "d981e334615d4a4d2ea003a4c3d59ee8f631d6f52532cfa70dc97143e97a6fe8",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinywallet-module-0.19.0-ubuntu-22.04-x86_64.tar.gz",
            "4511bf4a03f9163f5493472c1b294eccd8087e44efbde6b7e50c2ee4867b707c",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinywallet-module-0.19.0-ubuntu-24.04-arm64.tar.gz",
            "aa79e84f60892f2ff624f42619f928b2699bdbea613f7bc2a0595f769c7f1d68",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinywallet-module-0.19.0-ubuntu-24.04-x86_64.tar.gz",
            "55c27a4c85631aee0509e9c74ae1eceb5f3fb3aa43540294bdd4991eac5500af",
        ),
        (
            "windows-11-arm64",
            "tinywallet-module-0.19.0-windows-11-arm64.zip",
            "f91ef12a88c324a4c890be0ff8114239d835770541ce8cfd6059f015d3720764",
        ),
        (
            "windows-2022-x86_64",
            "tinywallet-module-0.19.0-windows-2022-x86_64.zip",
            "398bce97e33fa44faa3e92eb20cdd448089882233858a952b7710ad4d1806546",
        ),
        (
            "windows-2025-x86_64",
            "tinywallet-module-0.19.0-windows-2025-x86_64.zip",
            "17856ed57fb14840c3a4c5cde9167a68d9a8d578ee63728852b894d3d1e8fec1",
        ),
    ];
    actual.sort_unstable();
    published.sort_unstable();
    assert_eq!(actual, published);
}

#[test]
fn tinybox_registry_matches_the_published_v0124_release_manifest() {
    let record = find("tinybox").expect("compiled TinyBox module");
    assert_eq!(record.version, "0.1.24");
    assert_eq!(tinybox_bus::CONTRACT_VERSION, (1, 14));
    assert_eq!(
        record.release_url,
        "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.24"
    );

    let actual = record
        .assets
        .iter()
        .map(|asset| (asset.host_key, asset.archive, asset.sha256))
        .collect::<Vec<_>>();
    let published = [
        (
            "archlinux-arm64",
            "tinybox-0.1.24-archlinux-arm64.tar.gz",
            "11b4c959868e9e13f60af116946145530ca76ad68ab2ceef4fea11a0c2f76c5f",
        ),
        (
            "archlinux-x86_64",
            "tinybox-0.1.24-archlinux-x86_64.tar.gz",
            "84764f24331dfb5ff1798419e69ada288233506f6d28f4f582f5e6a87f65d9e9",
        ),
        (
            "fedora-43-arm64",
            "tinybox-0.1.24-fedora-43-arm64.tar.gz",
            "26763cb6a77e638adfb8641b58aaa0f57493403c7ab4dbca639f35e434fbd20c",
        ),
        (
            "fedora-43-x86_64",
            "tinybox-0.1.24-fedora-43-x86_64.tar.gz",
            "442d67e92c011125350e672a505925008b0bb26c368d3408c24029e60bc12be9",
        ),
        (
            "fedora-44-arm64",
            "tinybox-0.1.24-fedora-44-arm64.tar.gz",
            "67c0bf40748a1bf9e749bc918cdaddc5ba53f29f9cc3327a7c56fc82ef8dfdee",
        ),
        (
            "fedora-44-x86_64",
            "tinybox-0.1.24-fedora-44-x86_64.tar.gz",
            "5e320dcce68a3b36c61a33647575c9e3f1449388f489e6aa094b7684863ab377",
        ),
        (
            "macos-15-arm64",
            "tinybox-0.1.24-macos-15-arm64.tar.gz",
            "59a76b8ea5ce093d26616b115cd11493459dcefebec73c2b63fa115e4c70d6f7",
        ),
        (
            "macos-15-x86_64",
            "tinybox-0.1.24-macos-15-x86_64.tar.gz",
            "dd73c660e3c54004dad55257eeefc812fa8c2b3bf89d82e346b5bb4e95439b7f",
        ),
        (
            "macos-26-arm64",
            "tinybox-0.1.24-macos-26-arm64.tar.gz",
            "3c2091b15246cb6fb566c73cbfc2f1a334b9f1d1a4ea58839f3576536321b285",
        ),
        (
            "macos-26-x86_64",
            "tinybox-0.1.24-macos-26-x86_64.tar.gz",
            "a077fae96405163fe4cc2ae0dcdfb2dee9e2b8da8ea7f7122d180d0c2f3789eb",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinybox-0.1.24-ubuntu-22.04-arm64.tar.gz",
            "beb26115d378dce167ffe6970f57dfd268c9851f10e254306812be3f478f4567",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinybox-0.1.24-ubuntu-22.04-x86_64.tar.gz",
            "be69717def1d836a8e64e3ed544fdb4fb772ef641df68b473e9677f030f94809",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinybox-0.1.24-ubuntu-24.04-arm64.tar.gz",
            "4935e8099f1a373b7d3fffc6f5a4019d3e40a4c5c3d360e30c1d311ab6eba777",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinybox-0.1.24-ubuntu-24.04-x86_64.tar.gz",
            "33391c0e42a5dc48328bb3cb2f785c44f4fc056ec5fc125a655c1416669b233e",
        ),
        (
            "windows-11-arm64",
            "tinybox-0.1.24-windows-11-arm64.zip",
            "9b00a82bacbba00f528aaf6655bc005fa96751f774cb41cf960cb5c1ac0face9",
        ),
        (
            "windows-2022-x86_64",
            "tinybox-0.1.24-windows-2022-x86_64.zip",
            "a5b676f18bc1252ee81d614c35fd2b1ffb2b2ecc3a0335face170d2185432255",
        ),
        (
            "windows-2025-x86_64",
            "tinybox-0.1.24-windows-2025-x86_64.zip",
            "c09e9d417442be2748cea8f6a1fe17b323675400f3e8b82dc647ec5a84eb3b99",
        ),
    ];

    assert_eq!(actual.len(), 17);
    assert_eq!(actual, published);
}
#[test]
fn tinymcp_registry_matches_the_published_v0143_release_manifest() {
    let record = find("tinymcp").expect("compiled TinyMCP module");
    assert_eq!(record.version, "0.14.3");
    assert_eq!(tinymcp_bus::CONTRACT_VERSION, (1, 21));
    assert_eq!(
        record.release_url,
        "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.14.3"
    );
    let actual = record
        .assets
        .iter()
        .map(|asset| (asset.host_key, asset.archive, asset.sha256))
        .collect::<Vec<_>>();
    let published = [
        (
            "archlinux-rolling-arm64",
            "tinymcp-0.14.3-archlinux-arm64.tar.gz",
            "ce3429b277e8d6ff18e5cb2eae508681e282c5a662b04be6e5a6b0baef889309",
        ),
        (
            "archlinux-rolling-x86_64",
            "tinymcp-0.14.3-archlinux-x86_64.tar.gz",
            "e58fa20f6d4f5041951eeb8799050a251a01b5fde1a6442409ebec4ab0387e28",
        ),
        (
            "fedora-43-arm64",
            "tinymcp-0.14.3-fedora-43-arm64.tar.gz",
            "d792e3afd6b9bf715450a08b6dc0ea5640b4e31ec05824e73cabca742dc439d7",
        ),
        (
            "fedora-43-x86_64",
            "tinymcp-0.14.3-fedora-43-x86_64.tar.gz",
            "7d9e36f65058518e58a4a366797786c8c7cd9954070f9a9c2a38f4a4078d6bca",
        ),
        (
            "fedora-44-arm64",
            "tinymcp-0.14.3-fedora-44-arm64.tar.gz",
            "ae150372d30a8d90dd82c39a139db36322d03142509a8fa0aaa837227944d315",
        ),
        (
            "fedora-44-x86_64",
            "tinymcp-0.14.3-fedora-44-x86_64.tar.gz",
            "255559923bd7a2bcf8c7290cfe0d9eb39bde4cfeb99d35bb5ce3dd2c994b1819",
        ),
        (
            "macos-15-arm64",
            "tinymcp-0.14.3-macos-15-arm64.tar.gz",
            "5100539b489a0ea2e6b64334bd3857b6cf63af8d4c0f20d4efba3c675488d8a0",
        ),
        (
            "macos-15-x86_64",
            "tinymcp-0.14.3-macos-15-x86_64.tar.gz",
            "d485623e52b5ea1057ff1ad354c2338ee06666ba0ca4e941439e6b0a703e19da",
        ),
        (
            "macos-26-arm64",
            "tinymcp-0.14.3-macos-26-arm64.tar.gz",
            "f147c5fb4220a316e36ce9e68004166faa36b0f31d96f89cf154867b2d1cd13c",
        ),
        (
            "macos-26-x86_64",
            "tinymcp-0.14.3-macos-26-x86_64.tar.gz",
            "fb07cecdcf1c78076de4e985c089c17a3c7a78e2d3c7f35882eec5b8612c2418",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinymcp-0.14.3-ubuntu-22.04-arm64.tar.gz",
            "fc0b6c92683abf8b8d3c78983ce325d720482d23ecde60aaa606c5e6e0952e3b",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinymcp-0.14.3-ubuntu-22.04-x86_64.tar.gz",
            "e7f2eccefbc645c66916c4885658a1dc286f1eb72ec7a7546bbc9f37e4228946",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinymcp-0.14.3-ubuntu-24.04-arm64.tar.gz",
            "f1cd87ed2a3276a5e8bd483ca134ffb12c0638d94701be83dd54ddbab3f64c51",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinymcp-0.14.3-ubuntu-24.04-x86_64.tar.gz",
            "4858b6eb8d246ae3f18f0e3e5de02bf98f98c7cd9ca3489d2049200b09abf15e",
        ),
        (
            "windows-11-arm64",
            "tinymcp-0.14.3-windows-11-arm64.zip",
            "0ab17f2ceb5134984b13746f2c8ac2e04fa632d66faa691d61019d08f1470adc",
        ),
        (
            "windows-2022-x86_64",
            "tinymcp-0.14.3-windows-2022-x86_64.zip",
            "891e9be9218e4e67a6c6ce64f2ca593bd98dc13fb2885a5d2c5d1c787b371fe7",
        ),
        (
            "windows-2025-x86_64",
            "tinymcp-0.14.3-windows-2025-x86_64.zip",
            "82e6e42cd586bf5a2dcec7b56fe88b158e5078f4e26e55dd4154ef70467e89d2",
        ),
    ];
    assert_eq!(actual, published);
}

#[test]
fn ids_and_bus_names_are_unique() {
    // Two records claiming one bus name is a conflict tinybus would only
    // surface at load time, on whichever one happened to be second.
    for (i, record) in ALL.iter().enumerate() {
        for other in &ALL[i + 1..] {
            assert_ne!(record.id, other.id, "duplicate module id");
            assert_ne!(record.bus_name, other.bus_name, "duplicate bus name");
        }
    }
}

#[test]
fn every_object_path_matches_its_bus_name() {
    // tinybus derives a module's object path from its bus name by replacing
    // dots with slashes, and admission compares the two. A mismatch here is
    // a module that downloads and then refuses to load.
    for record in ALL {
        assert_eq!(
            record.object_path,
            format!("/{}", record.bus_name.replace('.', "/")),
            "{} object path does not match its bus name",
            record.id
        );
    }
}

#[test]
fn every_digest_is_a_lowercase_sha256() {
    // An uppercase or truncated digest is refused by tinybus at download
    // time, which is a slow way to find a typo in this file.
    for record in ALL {
        for asset in record.assets {
            assert_eq!(
                asset.sha256.len(),
                64,
                "{} / {} digest is not 64 characters",
                record.id,
                asset.host_key
            );
            assert!(
                asset
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                "{} / {} digest is not lowercase hex",
                record.id,
                asset.host_key
            );
        }
    }
}

#[test]
fn every_asset_name_carries_its_host_key_and_a_known_extension() {
    // tinybus selects the asset by exact name and requires a `.tar.gz` or
    // `.zip` archive. Most archive platform stems match their host keys;
    // TinyMCP's release uses the canonical `archlinux` stem for the rolling
    // Arch host keys.
    for record in ALL {
        for asset in record.assets {
            let archive_host_key = if record.id == "tinymcp" {
                asset
                    .host_key
                    .strip_prefix("archlinux-rolling-")
                    .map(|suffix| format!("archlinux-{suffix}"))
                    .unwrap_or_else(|| asset.host_key.to_owned())
            } else {
                asset.host_key.to_owned()
            };
            assert!(
                asset.archive.contains(&archive_host_key),
                "{} asset {} does not name its host key {}",
                record.id,
                asset.archive,
                asset.host_key
            );
            let windows = asset.host_key.starts_with("windows");
            assert_eq!(
                windows,
                asset.archive.ends_with(".zip"),
                "{} asset {} has the wrong archive format for its host",
                record.id,
                asset.archive
            );
            if !windows {
                assert!(asset.archive.ends_with(".tar.gz"));
            }
        }
    }
}

#[test]
fn every_asset_name_carries_the_pinned_version() {
    // The digests and the version have to describe one release; an asset
    // left behind at an older version would download bytes the digest
    // beside it never matched.
    for record in ALL {
        for asset in record.assets {
            assert!(
                asset.archive.contains(record.version),
                "{} asset {} is not from version {}",
                record.id,
                asset.archive,
                record.version
            );
        }
    }
}

#[test]
fn the_release_url_is_a_tag_on_github() {
    // tinybus refuses a URL that is not a tag, because a branch URL names
    // bytes that can change under a digest that was checked once.
    for record in ALL.iter().filter(|record| !record.assets.is_empty()) {
        assert!(
            record
                .release_url
                .starts_with("https://github.com/tinyhumansai/"),
            "{} release url is not an upstream GitHub URL",
            record.id
        );
        assert!(
            record.release_url.contains("/releases/tag/"),
            "{} release url is not a tag",
            record.id
        );
        assert!(
            record.release_url.ends_with(record.version),
            "{} release url does not name version {}",
            record.id,
            record.version
        );
    }
}

/// Every host key `platform` can produce, across the supported triples.
fn every_host_key() -> Vec<String> {
    let hosts = [
        ("linux", "x86_64", Some((2, 39))),
        ("linux", "aarch64", Some((2, 39))),
        ("linux", "x86_64", Some((2, 35))),
        ("linux", "aarch64", Some((2, 35))),
        ("macos", "x86_64", None),
        ("macos", "aarch64", None),
        ("windows", "x86_64", None),
        ("windows", "aarch64", None),
    ];
    let mut keys: Vec<String> = hosts
        .into_iter()
        .flat_map(|(os, arch, glibc)| candidates_for(os, arch, glibc))
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

fn supported_host_keys(_record: &super::ModuleRecord) -> Vec<String> {
    every_host_key()
}

#[test]
fn a_record_that_pins_a_release_covers_every_host_the_platform_table_offers() {
    // The two tables are written independently and would drift silently:
    // `platform` offering a key no release publishes turns a supported host
    // into an "unsupported host" at first use.
    //
    // Scoped to records that pin a release at all. A record with no assets
    // is a module this build knows but has no published artifact for; it
    // loads from a developer build or the module search path, and asserting
    // release coverage for a release that does not exist would only assert
    // that it does not exist. The partial-coverage case — the one that is
    // actually a bug — is caught below.
    for record in ALL.iter().filter(|record| !record.assets.is_empty()) {
        for key in supported_host_keys(record) {
            assert!(
                record.asset_for(&key).is_some(),
                "{} publishes no asset for {key}, which the platform table would ask for",
                record.id
            );
        }
    }
}

#[test]
fn a_record_publishes_for_every_host_or_for_none() {
    // Partial coverage is the drift that hurts: it looks supported until a
    // user on the missing platform reaches the feature. All-or-nothing keeps
    // "not published yet" distinguishable from "published and incomplete".
    for record in ALL {
        let host_keys = supported_host_keys(record);
        let covered = host_keys
            .into_iter()
            .filter(|key| record.asset_for(key).is_some())
            .count();
        assert!(
            covered == 0 || covered == supported_host_keys(record).len(),
            "{} publishes assets for {covered} of {} host keys",
            record.id,
            supported_host_keys(record).len()
        );
    }
}

#[test]
fn find_resolves_known_ids_only() {
    assert!(find("tinydocs").is_some());
    assert!(find("not-a-module").is_none());
}

#[test]
fn tinychannels_registry_matches_scoped_runtime_release() {
    let record = find("tinychannels").expect("compiled TinyChannels module");
    assert_eq!(record.version, "0.1.26");
    assert_eq!(tinychannels_bus::CONTRACT_VERSION, 16);
    assert_eq!(
        tinychannels_bus::module_relay::MODULE_OPERATION_VERSION,
        (1, 17)
    );
    let expected = [
        (
            "macos-15-arm64",
            "tinychannels-module-0.1.26-macos-15-arm64.tar.gz",
            "5924dc78a1b2931c526b5ca2590c675455eaca6db914c648a321ea9225d65142",
        ),
        (
            "macos-15-x86_64",
            "tinychannels-module-0.1.26-macos-15-x86_64.tar.gz",
            "9be7ad6b11e9dcf14cbe8dba246e8aab55da29b03b60c36cd1a81dc250e312ac",
        ),
        (
            "macos-26-arm64",
            "tinychannels-module-0.1.26-macos-26-arm64.tar.gz",
            "466d6ceaf91735d278e65280f315efa5cfec7e609c9fe12cb8ff4e545846a4e3",
        ),
        (
            "macos-26-x86_64",
            "tinychannels-module-0.1.26-macos-26-x86_64.tar.gz",
            "2f3636ac9bf0b5fadd4b2539730064437770dee25442991591842cf4f5e34d8c",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinychannels-module-0.1.26-ubuntu-22.04-arm64.tar.gz",
            "2456cb336de10471199ba38cc680b955242837b49e9b7ef72cf80a63516e834d",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinychannels-module-0.1.26-ubuntu-22.04-x86_64.tar.gz",
            "07fba7fb3d730cfebcc6e5fe8ecbc9c799e1d3b207cb3ad251d6579d965cd6b0",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinychannels-module-0.1.26-ubuntu-24.04-arm64.tar.gz",
            "141c7d769728070df7bce8bd1f8cf4e9e43501f55fa4d055750842d2f9da6734",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinychannels-module-0.1.26-ubuntu-24.04-x86_64.tar.gz",
            "ed507c285fd63b0b6ea377b5ae8840dda549a1e03442d95157ca9a5878fe7fa6",
        ),
        (
            "windows-11-arm64",
            "tinychannels-module-0.1.26-windows-11-arm64.zip",
            "834bb5d76b42d32840059ee40d0b1ee3c4aa59b03c0882994228a8013c6c6c56",
        ),
        (
            "windows-2022-x86_64",
            "tinychannels-module-0.1.26-windows-2022-x86_64.zip",
            "fc33d7cd4ebb2bc8b7dcacc33ab88d174d07d5b73e4c76eaa22bd74401033b08",
        ),
        (
            "windows-2025-x86_64",
            "tinychannels-module-0.1.26-windows-2025-x86_64.zip",
            "f0778807f16f022f7cebaba7f426d5453ed66313090be031418cf41743208938",
        ),
    ];
    assert_eq!(record.assets.len(), expected.len());
    for (key, archive, sha256) in expected {
        let asset = record.asset_for(key).expect("supported platform asset");
        assert_eq!(asset.archive, archive);
        assert_eq!(asset.sha256, sha256);
    }
}

#[cfg(feature = "voice")]
#[test]
fn voice_capture_release_pins_summary_contract_and_every_platform_digest() {
    let record = find("tinyvoice").unwrap();
    assert_eq!(record.version, "0.1.16");
    assert_eq!(tinyvoice_bus::CONTRACT_VERSION, (1, 5));
    let expected = [
        (
            "macos-15-arm64",
            "tinyvoice-module-0.1.16-macos-15-arm64.tar.gz",
            "72c75b371e72c22da9a149b0962395d74131b41910e2ed735d127667f9f38c1d",
        ),
        (
            "macos-15-x86_64",
            "tinyvoice-module-0.1.16-macos-15-x86_64.tar.gz",
            "a293f24b14570ac551be300d1aac28eddb0339066ab4ec0a231b6831fade4734",
        ),
        (
            "macos-26-arm64",
            "tinyvoice-module-0.1.16-macos-26-arm64.tar.gz",
            "8003a988fdc2eb4a6be724caf6d1ba16a97d3bbf7372921630bb180f02dfeb28",
        ),
        (
            "macos-26-x86_64",
            "tinyvoice-module-0.1.16-macos-26-x86_64.tar.gz",
            "20142bb6bb285b863b626d2d49c4879331de1f34cfc2847755d038adf5664abe",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinyvoice-module-0.1.16-ubuntu-22.04-arm64.tar.gz",
            "be42ffdf401d10bdaac083b05c42ebbeeca4f25649286cd0ada608fa02045c23",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinyvoice-module-0.1.16-ubuntu-22.04-x86_64.tar.gz",
            "9a43aedb59617d6322537c411f91710ffe31c9595066592e0abc6086aca13a1a",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinyvoice-module-0.1.16-ubuntu-24.04-arm64.tar.gz",
            "620167d61a331ad1260f3f432552cca57af4a0696b51b3e7d8d62ad1f0337a06",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinyvoice-module-0.1.16-ubuntu-24.04-x86_64.tar.gz",
            "90de2c9dce82e9d8e3864e9e0f827f1110599a3b5148d905a2841492145a1015",
        ),
        (
            "windows-11-arm64",
            "tinyvoice-module-0.1.16-windows-11-arm64.zip",
            "8272daea789109e6034ee3af5ad9a46e5559b511f9cf6c965c14225e1c59c69e",
        ),
        (
            "windows-2022-x86_64",
            "tinyvoice-module-0.1.16-windows-2022-x86_64.zip",
            "89fc25bdddbcb77393f8618aa4f7f00f9cd9abd27dd1e35ad184aafb4fe4fb9d",
        ),
        (
            "windows-2025-x86_64",
            "tinyvoice-module-0.1.16-windows-2025-x86_64.zip",
            "271ba8c8476c22535f9fba0086b0aa0a253f9d228cd6f6a73e91cbe24dfa8a2a",
        ),
    ];
    assert_eq!(record.assets.len(), expected.len());
    for (key, archive, digest) in expected {
        let asset = record.asset_for(key).unwrap();
        assert_eq!(asset.archive, archive);
        assert_eq!(asset.sha256, digest);
    }
}
