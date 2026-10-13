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
    assert_eq!(desktop.version, "0.13.0");
    assert_eq!(desktop.assets.len(), 17);
    assert_eq!(tinycomputer_bus::CONTRACT_VERSION, (2, 17));
    assert_eq!(
        desktop.asset_for("ubuntu-24.04-x86_64").unwrap().sha256,
        "b1e1d60c2e9d79153cb77831824e04b2b4d80297d0a4428ec99bcb0c5ca4c80e"
    );
    assert_eq!(
        desktop.asset_for("macos-26-arm64").unwrap().sha256,
        "c8a02a2f7e87ccabfea5f4c0d06c03e1785d3a5b7dc5885ec23ba459d23182da"
    );
}

#[test]
#[cfg(feature = "web3")]
fn tinywallet_registry_matches_the_verified_v022_release_manifest() {
    let wallet = find("tinywallet").expect("compiled TinyWallet module");
    assert_eq!(wallet.version, "0.22.0");
    #[cfg(feature = "web3")]
    assert_eq!(tinywallet_bus::CONTRACT_VERSION, (1, 14));
    assert_eq!(
        wallet.release_url,
        "https://github.com/tinyhumansai/tinywallet/releases/tag/v0.22.0"
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
            "tinywallet-module-0.22.0-archlinux-arm64.tar.gz",
            "c0a7911452b1bf1461c945d45170305122dcb791c0ea3378411b1332cc4e7013",
        ),
        (
            "archlinux-x86_64",
            "tinywallet-module-0.22.0-archlinux-x86_64.tar.gz",
            "f15aa217405a41a7d4b53462e2f9f3e3745184ec1de9e54f79deca54a4162177",
        ),
        (
            "fedora-43-arm64",
            "tinywallet-module-0.22.0-fedora-43-arm64.tar.gz",
            "c9a92837299aa2a43883341390ecceb54e2e8567c23fe9963be51077d9726c1e",
        ),
        (
            "fedora-43-x86_64",
            "tinywallet-module-0.22.0-fedora-43-x86_64.tar.gz",
            "a19486e72204f0150466e06372538f70b761ddd7ff49fab5bd736f7bc48581a4",
        ),
        (
            "fedora-44-arm64",
            "tinywallet-module-0.22.0-fedora-44-arm64.tar.gz",
            "5ac8b865224d229d3d7bd199cd651e3862d4658fd125aae4121a0a0e709edde2",
        ),
        (
            "fedora-44-x86_64",
            "tinywallet-module-0.22.0-fedora-44-x86_64.tar.gz",
            "d404e7e484d09835c843731ddbbc5a93354071d8ed900792e42f73e56c4ba13f",
        ),
        (
            "macos-15-arm64",
            "tinywallet-module-0.22.0-macos-15-arm64.tar.gz",
            "1a173add7bd67565583f0cd557584dd79b2d64b9330aca93a5ded27446fc24c7",
        ),
        (
            "macos-15-x86_64",
            "tinywallet-module-0.22.0-macos-15-x86_64.tar.gz",
            "99a6d43a5f9668b36c5d98ec51daf3c343515af870dbf4745abdc6cf4ef2cfe7",
        ),
        (
            "macos-26-arm64",
            "tinywallet-module-0.22.0-macos-26-arm64.tar.gz",
            "4da19b9a12677db4187ef2076a44341066bf96976dbe89845fff32ff7990b0dd",
        ),
        (
            "macos-26-x86_64",
            "tinywallet-module-0.22.0-macos-26-x86_64.tar.gz",
            "a00ec245f3b72810e7c25d513bb5976dc9ada07265f8bdec2217d217049c4399",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinywallet-module-0.22.0-ubuntu-22.04-arm64.tar.gz",
            "c37c0c477e89b38013cbe6e8d8f62ddff3cf08c94e0452f441ae041265f3e7ab",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinywallet-module-0.22.0-ubuntu-22.04-x86_64.tar.gz",
            "df6c0b4ad6b560242186357f10d29f490ac278fb2f94c6f80934c8723f64130d",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinywallet-module-0.22.0-ubuntu-24.04-arm64.tar.gz",
            "c31bf29e7f6004be268e484651621a8e2c9efc3221a3d91285318888079dbb41",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinywallet-module-0.22.0-ubuntu-24.04-x86_64.tar.gz",
            "d0ef3d6833c972c18a917f965b71e7df61087c6eb61496ff6af9ca76605325e2",
        ),
        (
            "windows-11-arm64",
            "tinywallet-module-0.22.0-windows-11-arm64.zip",
            "02969de77cd63c2d889dc65231de6327fa39b4b39ffbacee2c4524dc34f44512",
        ),
        (
            "windows-2022-x86_64",
            "tinywallet-module-0.22.0-windows-2022-x86_64.zip",
            "401cc74695b52a40806713ece3f31c971d23ee3cd0529b231aa36a809cb015a0",
        ),
        (
            "windows-2025-x86_64",
            "tinywallet-module-0.22.0-windows-2025-x86_64.zip",
            "880c52be4634c0693b64769d58462605075954430b36a7dc7fff7dc49ecdf4fe",
        ),
    ];
    actual.sort_unstable();
    published.sort_unstable();
    assert_eq!(actual, published);
}

#[test]
fn tinybox_registry_matches_the_published_v0129_release_manifest() {
    let record = find("tinybox").expect("compiled TinyBox module");
    assert_eq!(record.version, "0.1.29");
    assert_eq!(tinybox_bus::CONTRACT_VERSION, (1, 18));
    assert_eq!(
        record.release_url,
        "https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.29"
    );

    let actual = record
        .assets
        .iter()
        .map(|asset| (asset.host_key, asset.archive, asset.sha256))
        .collect::<Vec<_>>();
    let published = [
        (
            "archlinux-arm64",
            "tinybox-0.1.29-archlinux-arm64.tar.gz",
            "0461e2734aac0847ade5392f98369c2aa8ffa03ab85be326a819e2612183c998",
        ),
        (
            "archlinux-x86_64",
            "tinybox-0.1.29-archlinux-x86_64.tar.gz",
            "d033d073cad1143578eb3791b9c3dda020d54c83ce228d6278addf3eee5040fb",
        ),
        (
            "fedora-43-arm64",
            "tinybox-0.1.29-fedora-43-arm64.tar.gz",
            "6a30bb1d030de6b1a15bedc8d18e2693d6157f4d826af240da8dda0429be0b46",
        ),
        (
            "fedora-43-x86_64",
            "tinybox-0.1.29-fedora-43-x86_64.tar.gz",
            "5feba5818384b8cccf430ee8e0623eb8137a7ec5c4a5e51d0fbb741f83e59025",
        ),
        (
            "fedora-44-arm64",
            "tinybox-0.1.29-fedora-44-arm64.tar.gz",
            "b8befb06f3defb72aafa5b76995723a23ff76c580d96bf79a98ace8fb5b2ab62",
        ),
        (
            "fedora-44-x86_64",
            "tinybox-0.1.29-fedora-44-x86_64.tar.gz",
            "f3cf809688a4abdd5cbcdc5007762e0c82d625ff7851f0cb06895c162e52573f",
        ),
        (
            "macos-15-arm64",
            "tinybox-0.1.29-macos-15-arm64.tar.gz",
            "97f5a6f63001dc0612213e5ce393bb8903db2d6a9cff4be97d90e84afa9dd7c8",
        ),
        (
            "macos-15-x86_64",
            "tinybox-0.1.29-macos-15-x86_64.tar.gz",
            "45c3a48ebbeb8452bb7bab63d937fab8c70812d831805bc33636970ae94390ee",
        ),
        (
            "macos-26-arm64",
            "tinybox-0.1.29-macos-26-arm64.tar.gz",
            "12183a8fcb03ff149f550851c1c4e703a2b092c59a483726b37a501674613ec0",
        ),
        (
            "macos-26-x86_64",
            "tinybox-0.1.29-macos-26-x86_64.tar.gz",
            "4e74ad646b7fa4c31c619bc9680665b7bc30b06d4f3f42ac769856b195634454",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinybox-0.1.29-ubuntu-22.04-arm64.tar.gz",
            "50bb4b716a7f8f0db62a91a936f4fd4eeee03b96c0649e3e4fc9aa84e563f22c",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinybox-0.1.29-ubuntu-22.04-x86_64.tar.gz",
            "6672b0be315bcebef0eb255255981e034202f59e7223db6a1f8f539e20025a77",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinybox-0.1.29-ubuntu-24.04-arm64.tar.gz",
            "67afe1eed18edf16d832c7d1507bd47fb7dc863d4157edd0e5ecae2afa318b4c",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinybox-0.1.29-ubuntu-24.04-x86_64.tar.gz",
            "a51608744a06e579bc5e3a450521d32de78521162528b1e5005d55585eaf166c",
        ),
        (
            "windows-11-arm64",
            "tinybox-0.1.29-windows-11-arm64.zip",
            "5f096509db0a37a5f076d6d039ad4e8b2e3f092f66ab6484ec2f363321cd1932",
        ),
        (
            "windows-2022-x86_64",
            "tinybox-0.1.29-windows-2022-x86_64.zip",
            "1dba24ac470b79246f40fd888c3a62e4a51bab314f0d2d0104ba1ab5d48dd2c3",
        ),
        (
            "windows-2025-x86_64",
            "tinybox-0.1.29-windows-2025-x86_64.zip",
            "89ee0a8503410eee6a1591f730d169dc13102e1cea0873016100dcb35b1835d0",
        ),
    ];

    assert_eq!(actual.len(), 17);
    assert_eq!(actual, published);
}
#[test]
fn tinymcp_registry_matches_the_published_v0180_release_manifest() {
    let record = find("tinymcp").expect("compiled TinyMCP module");
    assert_eq!(record.version, "0.18.0");
    assert_eq!(tinymcp_bus::CONTRACT_VERSION, (1, 24));
    assert_eq!(
        record.release_url,
        "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.18.0"
    );
    let actual = record
        .assets
        .iter()
        .map(|asset| (asset.host_key, asset.archive, asset.sha256))
        .collect::<Vec<_>>();
    let published = [
        (
            "archlinux-rolling-arm64",
            "tinymcp-0.18.0-archlinux-arm64.tar.gz",
            "2926214fa8ea01a509f21264e9b39f6647bf9efd48655aa2b5e04f010eec4765",
        ),
        (
            "archlinux-rolling-x86_64",
            "tinymcp-0.18.0-archlinux-x86_64.tar.gz",
            "a2e3273a098325c935c6d64382a9a96bd96f2a0e557705d292e60dcd60886732",
        ),
        (
            "fedora-43-arm64",
            "tinymcp-0.18.0-fedora-43-arm64.tar.gz",
            "ab0345d9935546905181e2b29d4cbf1033c52c9ba5f027aa7a68b58d5fa64966",
        ),
        (
            "fedora-43-x86_64",
            "tinymcp-0.18.0-fedora-43-x86_64.tar.gz",
            "7618fb26992886ee53d94a8e67de725e2cc02b65cd4436989c5f76b7ed44c097",
        ),
        (
            "fedora-44-arm64",
            "tinymcp-0.18.0-fedora-44-arm64.tar.gz",
            "9502ef005fe2fff50d5c931f96e0fe496f04f8aaa5b7c42d845159c2ecd1d657",
        ),
        (
            "fedora-44-x86_64",
            "tinymcp-0.18.0-fedora-44-x86_64.tar.gz",
            "d73573e2a6481e9863db4ca4cac5c422c1dca8999fca99aca97f645471548865",
        ),
        (
            "macos-15-arm64",
            "tinymcp-0.18.0-macos-15-arm64.tar.gz",
            "f2704ff75b1399d2cc92d965d0c5cb9085b8e1f778b785f6ed6c9a184ea64f95",
        ),
        (
            "macos-15-x86_64",
            "tinymcp-0.18.0-macos-15-x86_64.tar.gz",
            "e7036da10f52af9248478e507897414b304c75b28e9156013124790ac4f97d23",
        ),
        (
            "macos-26-arm64",
            "tinymcp-0.18.0-macos-26-arm64.tar.gz",
            "e49a2d111988787e517694fb1b678e890da24cb77f4554bdce3d7ce973ea2d17",
        ),
        (
            "macos-26-x86_64",
            "tinymcp-0.18.0-macos-26-x86_64.tar.gz",
            "4fc5495b98ec119c0f939cafeeac8c393a0fe297dac5b384a21d7a7d82bb3800",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinymcp-0.18.0-ubuntu-22.04-arm64.tar.gz",
            "47c3934ca545dec412351fbd06f972c3d8ee9f9f87cec102d4073315b11aa5d2",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinymcp-0.18.0-ubuntu-22.04-x86_64.tar.gz",
            "259dd599a831a7e25c32687f9e1c8b9e7ebba0aec2bb34356921a4a35520ae98",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinymcp-0.18.0-ubuntu-24.04-arm64.tar.gz",
            "7fa57dd1f7b3d7585814c70e8b3dc65e79183fcc8cbda140ad68de80f858125a",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinymcp-0.18.0-ubuntu-24.04-x86_64.tar.gz",
            "3793da284a0698506c456d0117e0b3f774707c10ad53b7af65c2ab2d813153b4",
        ),
        (
            "windows-11-arm64",
            "tinymcp-0.18.0-windows-11-arm64.zip",
            "0479b54dd843ef0d1c0d048de76f10f5f09d0298247227ead83b41859d7828bd",
        ),
        (
            "windows-2022-x86_64",
            "tinymcp-0.18.0-windows-2022-x86_64.zip",
            "c16ea238bb7da168e0c96ddde69305e41df27a25a25073f7df4740d9b96a4f6d",
        ),
        (
            "windows-2025-x86_64",
            "tinymcp-0.18.0-windows-2025-x86_64.zip",
            "8899c643de2cd9274b06efc8dccbd13379bf7bbd570e87b276a315c0f1f84695",
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
    assert!(find("tinymcp").is_some());
    assert!(find("not-a-module").is_none());
}

#[test]
fn tinychannels_registry_matches_scoped_runtime_release() {
    let record = find("tinychannels").expect("compiled TinyChannels module");
    assert_eq!(record.version, "0.1.30");
    assert_eq!(tinychannels_bus::CONTRACT_VERSION, 21);
    assert_eq!(
        tinychannels_bus::module_relay::MODULE_OPERATION_VERSION,
        (1, 21)
    );
    let expected = [
        (
            "macos-15-arm64",
            "tinychannels-module-0.1.30-macos-15-arm64.tar.gz",
            "c23e3c099f33121e14a520dfec994e7b5da6205da02ea001d71b28889e5f269b",
        ),
        (
            "macos-15-x86_64",
            "tinychannels-module-0.1.30-macos-15-x86_64.tar.gz",
            "455e33ffb1d93b0a1023e8b4d528b49ccbc0ed95851b1fb2733649994efe9bfc",
        ),
        (
            "macos-26-arm64",
            "tinychannels-module-0.1.30-macos-26-arm64.tar.gz",
            "b2bcc666b980eab9a3090312ed83482a34591f1f4ecc73346f8e1e2422f482eb",
        ),
        (
            "macos-26-x86_64",
            "tinychannels-module-0.1.30-macos-26-x86_64.tar.gz",
            "d92cb8757f49ebaf3b5d7ecbd79af193a4d4bb002e9b78ce473091d649b08b31",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinychannels-module-0.1.30-ubuntu-22.04-arm64.tar.gz",
            "5852f6888541332cacaed2401ed67815588ba076d60926fdb31daedb13185b12",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinychannels-module-0.1.30-ubuntu-22.04-x86_64.tar.gz",
            "f63bb9034aa48a2335bea0a86ad0f1f920121281b88611c11a9e09fdac8cfe02",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinychannels-module-0.1.30-ubuntu-24.04-arm64.tar.gz",
            "810f68f723e9817d7343944137a00689ef997f57e4e241f496120350a1ba58f6",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinychannels-module-0.1.30-ubuntu-24.04-x86_64.tar.gz",
            "61e7a6f956b17ae53e83a2b0068f1009b0fde0b1266ca4f8f1a7feb8fb077104",
        ),
        (
            "windows-11-arm64",
            "tinychannels-module-0.1.30-windows-11-arm64.zip",
            "122b253a859f769a6ae5e89ea9adc5edaefc5471bf26e6bf8d09e7e0bef3d236",
        ),
        (
            "windows-2022-x86_64",
            "tinychannels-module-0.1.30-windows-2022-x86_64.zip",
            "2a8905d6a72bdbc7c4839fa7afb38362deef54eb2296f918b6af016dd8d54090",
        ),
        (
            "windows-2025-x86_64",
            "tinychannels-module-0.1.30-windows-2025-x86_64.zip",
            "feeddcb7beaa48e4bfda78e40cf6ce999cb7e4abd0414a6595da44665546af88",
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
    assert_eq!(record.version, "0.3.0");
    assert_eq!(tinyvoice_bus::CONTRACT_VERSION, (1, 10));
    let expected = [
        (
            "macos-15-arm64",
            "tinyvoice-module-0.3.0-macos-15-arm64.tar.gz",
            "91b231f6b6e48e3f78a9ae8b3fecab606c7d20f004fe300c030b883df845fd82",
        ),
        (
            "macos-15-x86_64",
            "tinyvoice-module-0.3.0-macos-15-x86_64.tar.gz",
            "f9f4ed3fa6e3615c7f747a619b8c32a2ee252d51541226e27142422273104dad",
        ),
        (
            "macos-26-arm64",
            "tinyvoice-module-0.3.0-macos-26-arm64.tar.gz",
            "4cdbab216808163d8fe33bf9c55eea1355fe1d84cd265167971dac2763f65807",
        ),
        (
            "macos-26-x86_64",
            "tinyvoice-module-0.3.0-macos-26-x86_64.tar.gz",
            "8f12b00fa3474cd5ba2baef3e0e8f8dfb30a401f103bf70d339d8d997ce79597",
        ),
        (
            "ubuntu-22.04-arm64",
            "tinyvoice-module-0.3.0-ubuntu-22.04-arm64.tar.gz",
            "d20f000a9343167016fbfef7007ee6fdd8b84f022cdc15d96c203a0b43aab058",
        ),
        (
            "ubuntu-22.04-x86_64",
            "tinyvoice-module-0.3.0-ubuntu-22.04-x86_64.tar.gz",
            "5e1f4b18a6662244fc1488166db153f9df60c651828e44e8bcfd55dc46698087",
        ),
        (
            "ubuntu-24.04-arm64",
            "tinyvoice-module-0.3.0-ubuntu-24.04-arm64.tar.gz",
            "fadbe5b4b9d47744a21096838c10e2ad03b3be43f690f8cef7deb1d2fc1bc427",
        ),
        (
            "ubuntu-24.04-x86_64",
            "tinyvoice-module-0.3.0-ubuntu-24.04-x86_64.tar.gz",
            "945fdd762be37c779bdf16b96da03109ce82c7b43d5bb9865e0dfc32e239d883",
        ),
        (
            "windows-11-arm64",
            "tinyvoice-module-0.3.0-windows-11-arm64.zip",
            "7b7e247fb2b6f8be6948ffbe21ed9244e01dac6f367db12b5a78d014d13b8ae9",
        ),
        (
            "windows-2022-x86_64",
            "tinyvoice-module-0.3.0-windows-2022-x86_64.zip",
            "b2485d846a3287c47da303237680e0cbe76a210e5ef2f25027220cde4ffb8ed4",
        ),
        (
            "windows-2025-x86_64",
            "tinyvoice-module-0.3.0-windows-2025-x86_64.zip",
            "ff91872dee9ac3ab098bea1830fe1bebd6f16a8cc3267e02ee36a74d0bc48469",
        ),
    ];
    assert_eq!(record.assets.len(), expected.len());
    for (key, archive, digest) in expected {
        let asset = record.asset_for(key).unwrap();
        assert_eq!(asset.archive, archive);
        assert_eq!(asset.sha256, digest);
    }
}

#[test]
fn every_record_names_a_source_manifest_that_exists_in_its_submodule() {
    let vendor = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    for record in ALL {
        let spec = &record.source;
        assert!(!spec.package.is_empty(), "{}: package", record.id);
        assert!(!spec.lib_name.is_empty(), "{}: lib_name", record.id);
        assert_eq!(spec.submodule, record.id, "{}: submodule dir", record.id);
        let manifest = vendor.join(spec.submodule).join(spec.manifest);
        assert!(
            manifest.is_file(),
            "{}: source manifest {} is missing (is the submodule initialised?)",
            record.id,
            spec.manifest
        );
    }
}

#[test]
fn source_artifact_file_names_follow_the_platform_convention() {
    for record in ALL {
        let name = record.source.artifact_filename();
        assert!(name.contains(record.source.lib_name), "{}", record.id);
        if cfg!(target_os = "linux") {
            assert!(name.starts_with("lib") && name.ends_with(".so"), "{name}");
        }
    }
}

#[test]
fn tinysearch_registry_matches_bus_contract_and_published_release() {
    let search = find("tinysearch").expect("compiled search module");
    assert_eq!(search.bus_name, tinysearch_bus::names::INTERFACE);
    assert_eq!(search.object_path, tinysearch_bus::names::OBJECT_PATH);
    assert_eq!(search.version, "0.5.0");
    assert_eq!(tinysearch_bus::CONTRACT_VERSION, (2, 2));
    assert_eq!(
        search.asset_for("ubuntu-24.04-x86_64").unwrap().sha256,
        "e038b2529e40d834f4754f3566aaafc5cb5c4a2f0b643a389b4b66ba7f91db74"
    );
}
