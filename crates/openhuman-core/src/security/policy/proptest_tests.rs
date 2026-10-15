//! Property-based tests for the autonomy classifier's adversarial-input
//! surfaces (plan.md §6.3 — proptest was previously absent). The command
//! classifier and path checks run on fully untrusted strings from the
//! LLM/user, so the central properties are "never panics on any input" and a
//! handful of fail-closed invariants that must hold for *all* inputs, not just
//! the hand-enumerated cases in `policy_tests.rs`.

use super::types::SecurityPolicy;
use proptest::prelude::*;
use tinybox_bus::CommandClass;

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// The string-level path check must never panic on arbitrary paths
    /// (traversal, NUL bytes, tilde, URL-encoding, unicode).
    #[test]
    fn path_string_check_never_panics(path in ".*") {
        let p = SecurityPolicy::default();
        let _ = p.is_path_string_allowed(&path);
    }
}
