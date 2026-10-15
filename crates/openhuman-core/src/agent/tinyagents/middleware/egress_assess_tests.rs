use super::*;

#[test]
fn non_sensitive_modes_never_refuse_or_assess() {
    for mode in [PrivacyMode::Standard, PrivacyMode::LocalOnly] {
        assert!(refusal_for(mode, "openai", "gpt", Some("Bearer abcdefghijklmnop")).is_none());
        assert!(refusal_for(mode, "openai", "gpt", None).is_none());
    }
}

#[test]
fn sensitive_refuses_a_request_the_module_cannot_assess() {
    // No armed module bridge here, so the answer is Unknown and fails closed.
    let message = refusal_for(PrivacyMode::Sensitive, "openai", "gpt", Some("hello"))
        .expect("an unassessed payload is refused");
    assert!(message.contains("Sensitive privacy mode"), "{message}");
    assert!(!message.contains("hello"), "{message}");
    // A request too large to assess is refused the same way.
    assert!(refusal_for(PrivacyMode::Sensitive, "openai", "gpt", None).is_some());
}
