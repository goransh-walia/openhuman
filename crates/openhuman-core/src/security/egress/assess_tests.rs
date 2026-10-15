use super::*;
use crate::security::egress::enforce::sensitive_blocks;
use crate::security::live_policy::test_privacy_scope;

#[test]
fn unchanged_payload_is_assessed_clean() {
    let assessment = assessment_from_answer("hello", Ok("hello".to_string()));
    assert_eq!(assessment.level, IdentificationRisk::None);
    assert!(assessment.categories.is_empty());
}

#[test]
fn changed_payload_is_high_risk_and_sensitive_refuses_it() {
    let assessment = assessment_from_answer("Bearer abc", Ok("[REDACTED]".to_string()));
    assert_eq!(assessment.level, IdentificationRisk::High);
    assert_eq!(assessment.categories, vec![CATEGORY_SENSITIVE_CONTENT]);
    let desc =
        EgressDescriptor::composio("TOOL").with_risk(assessment.level, assessment.categories);
    assert!(sensitive_blocks(PrivacyMode::Sensitive, &desc));
}

#[test]
fn no_module_answer_is_unassessed_and_sensitive_refuses_it() {
    let assessment = assessment_from_answer("hello", Err("not armed".to_string()));
    assert_eq!(assessment.level, IdentificationRisk::Unknown);
    let desc = EgressDescriptor::composio("TOOL");
    assert!(sensitive_blocks(PrivacyMode::Sensitive, &desc));
}

#[test]
fn clean_assessment_lets_sensitive_allow_a_content_transfer() {
    let desc = EgressDescriptor::composio("TOOL").with_risk(IdentificationRisk::None, Vec::new());
    assert!(!sensitive_blocks(PrivacyMode::Sensitive, &desc));
}

#[test]
fn other_modes_never_reach_the_module() {
    // A secret-looking payload under Standard stays unassessed: no module call is
    // made, so there is no answer to fold in, and Standard ignores the risk.
    for mode in [PrivacyMode::Standard, PrivacyMode::LocalOnly] {
        let assessment = assess_for_mode(mode, "Bearer abcdefghijklmnopqrstuvwxyz");
        assert_eq!(assessment.level, IdentificationRisk::Unknown, "{mode:?}");
    }
}

#[test]
fn assessed_leaves_the_descriptor_untouched_outside_sensitive() {
    let _scope = test_privacy_scope(PrivacyMode::Standard);
    let desc = EgressDescriptor::composio("TOOL").assessed("anything");
    assert_eq!(desc.risk_level, IdentificationRisk::Unknown);
    assert!(desc.risk_categories.is_empty());
    let parts = EgressDescriptor::composio("TOOL").assessed_parts(["a", "b"]);
    assert_eq!(parts.risk_level, IdentificationRisk::Unknown);
}

#[test]
fn rank_puts_an_unassessed_part_above_every_assessed_one() {
    let order = [
        IdentificationRisk::None,
        IdentificationRisk::Low,
        IdentificationRisk::Medium,
        IdentificationRisk::High,
        IdentificationRisk::Unknown,
    ];
    for pair in order.windows(2) {
        assert!(rank(pair[0]) < rank(pair[1]), "{pair:?}");
    }
}

/// Sensitive mode with the real module: a clean payload is assessed clean and
/// permitted, a credential-bearing one is High and refused, and an unarmed
/// bridge elsewhere stays Unknown (covered above).
#[cfg(feature = "security-module")]
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key (downloads the published archive)"]
async fn sensitive_assessment_on_the_released_archive() {
    let (_dir, config) = crate::security::redact::tests::released_config();
    crate::security::redact::tests::arm(&config).await;

    let _scope = test_privacy_scope(PrivacyMode::Sensitive);
    let clean = EgressDescriptor::composio("TOOL").assessed("summarise the quarterly plan");
    assert_eq!(clean.risk_level, IdentificationRisk::None);
    assert!(!sensitive_blocks(PrivacyMode::Sensitive, &clean));

    let secret = format!(
        "send Authorization: Bearer {}",
        "abcdefghijklmnopqrstuvwxyz0123456789"
    );
    let dirty = EgressDescriptor::composio("TOOL").assessed(&secret);
    assert_eq!(dirty.risk_level, IdentificationRisk::High);
    assert!(sensitive_blocks(PrivacyMode::Sensitive, &dirty));

    let batch =
        EgressDescriptor::embedding("cloud", "m").assessed_parts(["fine text", secret.as_str()]);
    assert_eq!(batch.risk_level, IdentificationRisk::High);
}
