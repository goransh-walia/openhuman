use super::*;
use super::skill_scan::scan_bundle_snapshot;
use tinyskills::Verdict as ScanVerdict;

#[test]
fn scan_snapshot_binds_document_and_resource_content() {
    let clean = scan_bundle_snapshot("---\nname: demo\ndescription: safe\n---\nbody", &[]);
    let changed_document = scan_bundle_snapshot("---\nname: demo\ndescription: changed\n---\nbody", &[]);
    let changed_resource = scan_bundle_snapshot(
        "---\nname: demo\ndescription: safe\n---\nbody",
        &[("references/guide.md", "resource")],
    );

    assert_ne!(clean.bundle_sha256, changed_document.bundle_sha256);
    assert_ne!(clean.bundle_sha256, changed_resource.bundle_sha256);
}

#[test]
fn scanner_verdict_covers_the_document_and_bundled_resources() {
    let document_block = scan_bundle_snapshot(
        "---\nname: demo\ndescription: safe\n---\nbody\u{200b}",
        &[],
    );
    let resource_block = scan_bundle_snapshot(
        "---\nname: demo\ndescription: safe\n---\nbody",
        &[("references/guide.md", "safe\u{200b}text")],
    );

    assert_eq!(document_block.verdict, ScanVerdict::Block);
    assert_eq!(resource_block.verdict, ScanVerdict::Block);
}
