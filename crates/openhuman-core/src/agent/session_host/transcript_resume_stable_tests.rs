//! A workspace written by the previous release resumes in place: resuming a
//! committed fixture neither renames, rewrites nor re-adopts anything, so the
//! transcript stems and bytes on disk are exactly what the old binary left.

use std::collections::BTreeMap;
use std::path::Path;

use tinyinference_llm::model::ModelResponse;

use super::transcript_compat_tests::{
    build_host, model, place_fixture, run_async, session_raw, thread_id, Layout, SCENARIOS,
};

/// File name -> bytes for everything under the workspace's `session_raw/`.
fn raw_tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let Ok(entries) = std::fs::read_dir(session_raw(root)) else {
        return out;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        out.insert(name, std::fs::read(entry.path()).expect("read raw file"));
    }
    out
}

#[test]
fn resuming_a_committed_workspace_leaves_every_stem_and_byte_untouched() {
    for scenario in SCENARIOS
        .iter()
        .filter(|scenario| scenario.layout != Layout::Legacy)
    {
        run_async(async move {
            let root = tempfile::tempdir().expect("tempdir");
            let next = model(vec![ModelResponse::assistant("unused")], scenario.native);
            let mut host = build_host(
                root.path(),
                next,
                scenario.native,
                &thread_id(scenario.name),
            );
            let stem = host.session_id().expect("bound session id");
            place_fixture(root.path(), scenario, &stem, "");
            let before = raw_tree(root.path());
            assert!(!before.is_empty(), "{}: fixture placed", scenario.name);

            assert!(host.resume_bound_session().await.expect("resume"));

            assert_eq!(
                before,
                raw_tree(root.path()),
                "{}: resume must not rename, create or rewrite transcript files",
                scenario.name
            );
        });
    }
}

#[test]
fn a_legacy_workspace_is_adopted_once_and_the_next_process_resumes_in_place() {
    let scenario = SCENARIOS
        .iter()
        .find(|scenario| scenario.layout == Layout::Legacy)
        .expect("legacy scenario");
    run_async(async move {
        let root = tempfile::tempdir().expect("tempdir");
        let thread = thread_id(scenario.name);
        let legacy_only;
        let after_first;
        {
            let next = model(vec![ModelResponse::assistant("unused")], scenario.native);
            let mut host = build_host(root.path(), next, scenario.native, &thread);
            let stem = host.session_id().expect("bound session id");
            place_fixture(root.path(), scenario, &stem, "");
            legacy_only = raw_tree(root.path());
            assert!(host.resume_bound_session().await.expect("first resume"));
            after_first = raw_tree(root.path());
        }
        // Adoption writes the session's own file and modifies no legacy file.
        for (name, bytes) in &legacy_only {
            assert_eq!(
                after_first.get(name),
                Some(bytes),
                "legacy file {name} must stay byte-for-byte"
            );
        }
        assert!(
            after_first.len() > legacy_only.len(),
            "adopted file written"
        );

        // A fresh process over the same workspace resumes without adopting again.
        let next = model(vec![ModelResponse::assistant("unused")], scenario.native);
        let mut host = build_host(root.path(), next, scenario.native, &thread);
        assert!(host.resume_bound_session().await.expect("second resume"));
        assert_eq!(
            after_first,
            raw_tree(root.path()),
            "second resume must not re-adopt or rewrite anything"
        );
    });
}
