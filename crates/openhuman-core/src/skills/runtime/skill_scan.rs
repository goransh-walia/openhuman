//! Host adapter for scanning a workflow bundle before TinySecurity evaluates it.
//!
//! The digest binds the verdict to the exact document and resource snapshot the
//! caller will expose. Resource ordering and length-prefixing make the digest
//! unambiguous and stable across platforms.

use sha2::{Digest, Sha256};
use tinyskills::{ScanDocument, ScanReport, ScanResource, Verdict};

/// Scan verdict and content identity for one workflow bundle snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SkillScanSnapshot {
    pub(crate) verdict: Verdict,
    pub(crate) bundle_sha256: String,
}

/// Scan the document and resources and bind the result to their exact content.
pub(crate) fn scan_bundle_snapshot(
    document: &str,
    resources: &[(&str, &str)],
) -> SkillScanSnapshot {
    let mut resources = resources
        .iter()
        .map(|(path, text)| ScanResource {
            path: (*path).to_owned(),
            text: (*text).to_owned(),
        })
        .collect::<Vec<_>>();
    resources.sort_by(|left, right| left.path.cmp(&right.path));

    let report: ScanReport = tinyskills::scan_skill(
        &ScanDocument {
            name: "",
            description: "",
            category: None,
            version: None,
            body: document,
            extra_frontmatter: &[],
        },
        &resources,
    );

    let mut hasher = Sha256::new();
    hasher.update(b"openhuman.skill-bundle.v1\0");
    hash_field(&mut hasher, b"document");
    hash_field(&mut hasher, document.as_bytes());
    for resource in &resources {
        hash_field(&mut hasher, b"resource-path");
        hash_field(&mut hasher, resource.path.as_bytes());
        hash_field(&mut hasher, b"resource-content");
        hash_field(&mut hasher, resource.text.as_bytes());
    }

    SkillScanSnapshot {
        verdict: report.verdict(),
        bundle_sha256: hex::encode(hasher.finalize()),
    }
}

fn hash_field(hasher: &mut Sha256, field: &[u8]) {
    hasher.update((field.len() as u64).to_be_bytes());
    hasher.update(field);
}
