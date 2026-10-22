use super::*;

#[test]
fn configured_openhuman_jwt_slug_routes_to_managed_chat_model() {
    let _guard = crate::inference::inference_test_guard();
    let mut config = Config::default();
    config.cloud_providers.push(oh_entry("p_oh"));
    // A retired tier slug after the managed slug is a role alias: it runs on
    // the managed default, never reaches the backend verbatim.
    config.chat_provider = Some("openhuman:reasoning-v1".to_string());

    let (model, model_id) =
        try_create_cloud_slug_chat_model("chat", &config, &EgressLedger::none())
            .expect("configured OpenhumanJwt slug should be recognized")
            .expect("managed model should build");

    assert_eq!(model_id, crate::config::MODEL_MANAGED_DEFAULT);

    // A concrete catalog id is forwarded verbatim.
    config.chat_provider = Some("openhuman:openrouter/deepseek/deepseek-v4-pro".to_string());
    let (_, pinned_id) = try_create_cloud_slug_chat_model("chat", &config, &EgressLedger::none())
        .expect("configured OpenhumanJwt slug should be recognized")
        .expect("managed model should build");
    assert_eq!(pinned_id, "openrouter/deepseek/deepseek-v4-pro");
    assert_eq!(
        model
            .profile()
            .and_then(|profile| profile.provider.as_deref()),
        Some("managed"),
        "OpenhumanJwt must use the crate-native managed backend model"
    );
}

#[tokio::test]
async fn openhuman_jwt_slug_discloses_pinned_model() {
    use crate::core::events::DomainEvent;
    use crate::security::egress::{EgressDescriptor, EgressReason};
    use std::time::Duration;

    let _guard = crate::inference::inference_test_guard_async().await;
    crate::core::bus::init().await.expect("bus init");
    let mut rx = crate::core::bus::BUS.get().unwrap().receiver();

    let marker = "egress-jwt-pinned-marker-v1";
    let mut config = Config::default();
    config.cloud_providers.push(oh_entry("p_oh"));
    let provider = format!("openhuman:{marker}");
    let _ = try_create_cloud_slug_chat_model_from_string(
        "chat",
        &provider,
        &config,
        &EgressLedger::none(),
    )
    .expect("configured OpenhumanJwt slug should be recognized")
    .expect("managed model should build");

    let sentinel = "egress-jwt-pinned-sentinel-end";
    crate::core::bus::BUS.publish(DomainEvent::ExternalTransferPending {
        descriptor: EgressDescriptor::network_fetch(sentinel),
        thread_id: None,
        client_id: None,
        request_id: None,
    });

    let mut count = 0usize;
    loop {
        match tokio::time::timeout(Duration::from_secs(3), rx.recv()).await {
            Ok(Some(DomainEvent::ExternalTransferPending { descriptor, .. })) => {
                if descriptor.service == marker {
                    assert_eq!(descriptor.provider_slug, "openhuman");
                    assert!(matches!(descriptor.reason, EgressReason::Inference));
                    count += 1;
                } else if descriptor.service == sentinel {
                    break;
                }
            }
            Ok(Some(_)) => continue,
            Ok(None) => panic!("the bus closed before the sentinel arrived"),
            Err(_) => panic!("timed out before egress sentinel arrived"),
        }
    }
    assert_eq!(
        count, 1,
        "JWT construction must disclose its pinned model once"
    );
}

#[tokio::test]
async fn native_claude_turn_routes_disclose_pinned_models() {
    use crate::core::bus::BUS;
    use crate::core::events::DomainEvent;
    use crate::security::egress::EgressDescriptor;
    use std::time::Duration;

    let _guard = crate::inference::inference_test_guard_async().await;
    crate::core::bus::init().await.expect("bus init");
    let mut rx = crate::core::bus::BUS.get().unwrap().receiver();

    let configured_sdk = "egress-sdk-configured-marker";
    let pinned_sdk = "egress-sdk-pinned-marker";
    let mut sdk_config = Config::default();
    sdk_config.chat_provider = Some(format!("claude_agent_sdk:{configured_sdk}"));
    create_turn_chat_model(
        &crate::core::runtime::DispatchCtx::ambient(),
        "chat",
        &sdk_config,
        pinned_sdk,
        0.0,
    )
    .expect("Claude Agent SDK turn model should build");

    let configured_code = "egress-code-configured-marker";
    let pinned_code = "egress-code-pinned-marker";
    let mut code_config = Config::default();
    code_config.chat_provider = Some(format!("claude-code:{configured_code}"));
    // Egress is disclosed once the effective model is selected, before the
    // environment probe. The test therefore remains valid on hosts without the
    // Claude Code CLI.
    let _ = create_turn_chat_model(
        &crate::core::runtime::DispatchCtx::ambient(),
        "chat",
        &code_config,
        pinned_code,
        0.0,
    );

    let sentinel = "egress-native-claude-sentinel-end";
    BUS.publish(DomainEvent::ExternalTransferPending {
        descriptor: EgressDescriptor::network_fetch(sentinel),
        thread_id: None,
        client_id: None,
        request_id: None,
    });

    let mut sdk_count = 0usize;
    let mut code_count = 0usize;
    let mut configured_count = 0usize;
    loop {
        match tokio::time::timeout(Duration::from_secs(3), rx.recv()).await {
            Ok(Some(DomainEvent::ExternalTransferPending { descriptor, .. })) => {
                match descriptor.service.as_str() {
                    service if service == pinned_sdk => {
                        assert_eq!(descriptor.provider_slug, "claude_agent_sdk");
                        sdk_count += 1;
                    }
                    service if service == pinned_code => {
                        assert_eq!(descriptor.provider_slug, "claude-code");
                        code_count += 1;
                    }
                    service if service == configured_sdk || service == configured_code => {
                        configured_count += 1;
                    }
                    service if service == sentinel => break,
                    _ => {}
                }
            }
            Ok(Some(_)) => continue,
            Ok(None) => panic!("the bus closed before the sentinel arrived"),
            Err(_) => panic!("timed out before egress sentinel arrived"),
        }
    }

    assert_eq!(
        sdk_count, 1,
        "SDK route must disclose its pinned model once"
    );
    assert_eq!(
        code_count, 1,
        "Claude Code route must disclose its pinned model once"
    );
    assert_eq!(
        configured_count, 0,
        "native Claude routes must not disclose stale configured models"
    );
}

#[test]
fn openhuman_jwt_slug_preserves_forced_text_mode() {
    let _guard = crate::inference::inference_test_guard();
    let mut config = Config::default();
    config.cloud_providers.push(oh_entry("p_oh"));

    let (model, _) = try_create_cloud_slug_chat_model_from_string_with_native_tools(
        "chat",
        "openhuman:reasoning-v1",
        &config,
        false,
        &EgressLedger::none(),
    )
    .expect("configured OpenhumanJwt slug should be recognized")
    .expect("managed model should build");

    let profile = model
        .profile()
        .expect("managed model should expose its effective capabilities");
    assert!(!profile.tool_calling);
    assert!(!profile.parallel_tool_calls);
    assert!(!profile.streaming_tool_chunks);
}

#[test]
fn openhuman_jwt_slug_without_model_preserves_managed_role_tier() {
    let _guard = crate::inference::inference_test_guard();
    let mut config = Config::default();
    config.cloud_providers.push(oh_entry("p_oh"));

    let (_model, model_id) = try_create_cloud_slug_chat_model_from_string(
        "summarization",
        "openhuman:",
        &config,
        &EgressLedger::none(),
    )
    .expect("configured OpenhumanJwt slug should be recognized")
    .expect("managed model should build");

    assert_eq!(model_id, crate::config::MODEL_MANAGED_DEFAULT);
}

#[test]
fn try_create_cloud_slug_flips_openai_but_declines_non_cloud() {
    let _guard = crate::inference::inference_test_guard();
    // `openai` (API-key Bearer, no codex OAuth) now flips crate-native on Chat
    // Completions — the legacy `/v1/responses` fallback is not replicated.
    let mut openai = Config::default();
    openai.cloud_providers.push(openai_entry("p_oai", "openai"));
    openai.chat_provider = Some("openai:gpt-4o-mini".to_string());
    let (model, model_id) =
        try_create_cloud_slug_chat_model("chat", &openai, &EgressLedger::none())
            .expect("openai should flip crate-native")
            .expect("build");
    assert_eq!(model_id, "gpt-4o-mini");
    assert_eq!(
        model.profile().and_then(|p| p.provider.as_deref()),
        Some("openai")
    );

    // Managed (default), local runtimes, and unconfigured slugs are not cloud
    // slugs — they decline and fall through to their own paths.
    assert!(
        try_create_cloud_slug_chat_model("chat", &Config::default(), &EgressLedger::none())
            .is_none()
    );
    let mut local = Config::default();
    local.chat_provider = Some("ollama:qwen2.5".to_string());
    assert!(try_create_cloud_slug_chat_model("chat", &local, &EgressLedger::none()).is_none());
    let mut unconfigured = Config::default();
    unconfigured.chat_provider = Some("deepseek:deepseek-chat".to_string());
    assert!(
        try_create_cloud_slug_chat_model("chat", &unconfigured, &EgressLedger::none()).is_none()
    );
}
