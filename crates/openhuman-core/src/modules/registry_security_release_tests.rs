use super::find;
#[test]
fn tinysecurity_registry_matches_the_published_v017_release_manifest() {
    let security = find("tinysecurity").expect("compiled TinySecurity module");
    assert_eq!(security.version, "0.17.0");
    assert_eq!(
        security.release_url,
        "https://github.com/tinyhumansai/tinysecurity/releases/tag/v0.17.0"
    );
    assert_eq!(
        tinysecurity_bus::CONTRACT_VERSION,
        (1, 27),
        "the checked-out bus contract matches the published module"
    );

    let actual = security
        .assets
        .iter()
        .map(|asset| (asset.host_key, asset.archive, asset.sha256))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            (
                "ubuntu-24.04-x86_64",
                "tinysecurity-module-0.17.0-ubuntu-24.04-x86_64.tar.gz",
                "37d05c3d7b8cea819c1879dda17c401c4ade217f9cb2409aacd0e5ef5b3a18b9",
            ),
            (
                "ubuntu-24.04-arm64",
                "tinysecurity-module-0.17.0-ubuntu-24.04-arm64.tar.gz",
                "663928e698d620ea95b08e5af872e34c2b8d000e796e0e1d3cb75ea8b809ba6e",
            ),
            (
                "ubuntu-22.04-x86_64",
                "tinysecurity-module-0.17.0-ubuntu-22.04-x86_64.tar.gz",
                "e1d5c9622cc0c647583a6ed3ccb38498c276725f1ef8f8d3c304988308c3b5e4",
            ),
            (
                "ubuntu-22.04-arm64",
                "tinysecurity-module-0.17.0-ubuntu-22.04-arm64.tar.gz",
                "c7d90ff3cfcdcf9a0d6574530f202ea95c411a9eac1072349c1de77a1b426b4a",
            ),
            (
                "macos-26-arm64",
                "tinysecurity-module-0.17.0-macos-26-arm64.tar.gz",
                "2b2c9f40b488c5b9d366d78b6bca22444d748fd248826d7ca06bbd16c46004c9",
            ),
            (
                "macos-26-x86_64",
                "tinysecurity-module-0.17.0-macos-26-x86_64.tar.gz",
                "9a54afed108d45906753999d4aa7088af3b6fc0fe129e742216cacba45b0c49e",
            ),
            (
                "macos-15-arm64",
                "tinysecurity-module-0.17.0-macos-15-arm64.tar.gz",
                "0375fb6bad3ac9429b714c0110b1e6103c9ba8b407fc24061e5477fadcf28ed2",
            ),
            (
                "macos-15-x86_64",
                "tinysecurity-module-0.17.0-macos-15-x86_64.tar.gz",
                "142a8b33263a6a6e2ffc01c522a290fca202355c284fb5b7cee35f77e62f4b71",
            ),
            (
                "windows-2025-x86_64",
                "tinysecurity-module-0.17.0-windows-2025-x86_64.zip",
                "47052cc51af772c362fd34c97c6cb1fda86485bd73acea0a0c38b83b077533ce",
            ),
            (
                "windows-2022-x86_64",
                "tinysecurity-module-0.17.0-windows-2022-x86_64.zip",
                "6a75d7363ef698dbd2cbf8c7ce5bfc11b7b36064ef694730a1eb0ff76fdf0442",
            ),
            (
                "windows-11-arm64",
                "tinysecurity-module-0.17.0-windows-11-arm64.zip",
                "84848e023e7d09941cb27af567fd29951b7b9ce250562b2e91cb0b8241559ff9",
            ),
        ]
    );
}
