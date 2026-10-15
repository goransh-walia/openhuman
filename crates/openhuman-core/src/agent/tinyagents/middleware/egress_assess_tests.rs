use super::*;

fn assessed(level: IdentificationRisk, categories: &[&str]) -> Option<Assessment> {
    Some(Assessment {
        level,
        categories: categories.iter().map(|c| c.to_string()).collect(),
    })
}

#[test]
fn non_sensitive_modes_never_refuse() {
    for mode in [PrivacyMode::Standard, PrivacyMode::LocalOnly] {
        assert!(refusal_with(
            mode,
            "openai",
            "gpt",
            assessed(IdentificationRisk::High, &["credential_or_identifier"])
        )
        .is_none());
        assert!(refusal_with(mode, "openai", "gpt", None).is_none());
    }
}

#[test]
fn sensitive_permits_a_request_assessed_clean() {
    assert!(refusal_with(
        PrivacyMode::Sensitive,
        "openai",
        "gpt",
        assessed(IdentificationRisk::None, &[])
    )
    .is_none());
}

#[test]
fn sensitive_refuses_identifying_unassessed_and_missing_assessments() {
    for assessment in [
        assessed(IdentificationRisk::High, &["credential_or_identifier"]),
        assessed(IdentificationRisk::Unknown, &[]),
        None,
    ] {
        let message =
            refusal_with(PrivacyMode::Sensitive, "openai", "gpt", assessment).expect("refused");
        assert!(message.contains("Sensitive privacy mode"), "{message}");
    }
}
