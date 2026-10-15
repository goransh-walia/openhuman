//! The walk of a Composio outcome around the module's `RedactWith`: every string
//! channel is sent for checking, answers are written back (object keys included,
//! colliding keys kept apart), and a result the module could not check is
//! withheld. What the module removes is pinned against the released archive in
//! `modules/security_text_tests.rs`.

use std::collections::HashMap;

use serde_json::json;

use crate::config::{ComposioHostCredential, Config};
use crate::integrations::composio::tools::redact::{
    composio_secrets, redact_and_report, redact_composio_outcome, redact_outcome_with, Answers,
    WITHHELD,
};

const SHORT_KEY: &str = "abcdefgh";
const LONG_KEY: &str = "abcdefghi";

fn keys() -> Vec<String> {
    vec![LONG_KEY.to_string(), SHORT_KEY.to_string()]
}

fn config_with_overlapping_secrets() -> Config {
    let tmp = tempfile::tempdir().expect("tempdir for config_with_overlapping_secrets");
    let mut config = Config::default();
    config.config_path = tmp.path().join("config.toml");
    config.composio.api_key = Some(LONG_KEY.to_string());
    config
        .composio
        .pin_host_credential(ComposioHostCredential::direct(SHORT_KEY));
    std::mem::forget(tmp);
    config
}

/// A stand-in for the module: replaces the exact secrets, longest first, like
/// the module's registry does.
async fn replace_longest_first(strings: Vec<String>) -> Answers {
    let secrets = [LONG_KEY, SHORT_KEY];
    Ok(strings
        .into_iter()
        .map(|text| {
            let redacted = secrets
                .iter()
                .fold(text.clone(), |acc, key| acc.replace(key, "[REDACTED]"));
            (text, redacted)
        })
        .collect())
}

async fn cannot_answer(_strings: Vec<String>) -> Answers {
    Err("TinySecurity RedactWith failed".to_string())
}

fn texts(result: &tinytools::ToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| match c {
            tinytools::ToolContent::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_host_resolves_every_key_a_call_can_use_longest_first() {
    let secrets = composio_secrets(&config_with_overlapping_secrets());
    assert_eq!(secrets, vec![LONG_KEY.to_string(), SHORT_KEY.to_string()]);
}

#[tokio::test]
async fn overlapping_secrets_redact_longest_first_in_text() {
    let outcome = redact_outcome_with(
        Ok(tinytools::ToolResult::success(format!(
            "token {LONG_KEY} in transit"
        ))),
        &keys(),
        replace_longest_first,
    )
    .await
    .unwrap();
    assert_eq!(texts(&outcome), "token [REDACTED] in transit");
}

/// A secret that happened to be used as a JSON object key must not survive.
#[tokio::test]
async fn secret_used_as_a_json_object_key_is_redacted() {
    let outcome = redact_outcome_with(
        Ok(tinytools::ToolResult::json(
            json!({ format!("token_{LONG_KEY}"): "value" }),
        )),
        &keys(),
        replace_longest_first,
    )
    .await
    .unwrap();
    let serialized = serde_json::to_string(&outcome).unwrap();
    assert!(!serialized.contains(LONG_KEY), "{serialized}");
    assert!(serialized.contains("[REDACTED]"), "{serialized}");
}

#[tokio::test]
async fn colliding_redacted_object_keys_keep_both_values() {
    let outcome = redact_outcome_with(
        Ok(tinytools::ToolResult::json(json!({
            (SHORT_KEY): "short value",
            (LONG_KEY): "long value",
        }))),
        &keys(),
        replace_longest_first,
    )
    .await
    .unwrap();
    let tinytools::ToolContent::Json { data } = &outcome.content[0] else {
        panic!("expected JSON content");
    };
    let object = data.as_object().expect("JSON object");
    assert_eq!(object.len(), 2);
    let mut values: Vec<_> = object.values().map(|v| v.as_str().unwrap()).collect();
    values.sort_unstable();
    assert_eq!(values, vec!["long value", "short value"]);
    assert!(object.keys().all(|key| key.starts_with("[REDACTED]")));
}

/// Only a string that mentions a key is sent for checking, so a result that
/// never mentions one costs no module call and cannot be withheld.
#[tokio::test]
async fn only_strings_that_mention_a_key_are_sent_to_the_module() {
    use std::sync::Mutex;
    let seen = Mutex::new(Vec::<Vec<String>>::new());
    let redactor = |strings: Vec<String>| {
        seen.lock().unwrap().push(strings.clone());
        async move {
            Ok(strings
                .into_iter()
                .map(|s| (s.clone(), s.replace(LONG_KEY, "[REDACTED]")))
                .collect::<HashMap<_, _>>())
        }
    };
    let result = redact_outcome_with(
        Ok(tinytools::ToolResult::json(
            json!({"clean": "a long enough value", "dirty": format!("has {LONG_KEY} inside")}),
        )),
        &keys(),
        &redactor,
    )
    .await
    .unwrap();
    let serialized = serde_json::to_string(&result).unwrap();
    assert!(serialized.contains("a long enough value"));
    assert!(!serialized.contains(LONG_KEY), "{serialized}");
    let sent = seen.lock().unwrap().clone();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0], vec![format!("has {LONG_KEY} inside")]);

    // A result that never mentions a key does not reach the module at all, so a
    // module that cannot answer cannot withhold it.
    let clean = redact_outcome_with(
        Ok(tinytools::ToolResult::success("nothing sensitive here")),
        &keys(),
        cannot_answer,
    )
    .await
    .unwrap();
    assert_eq!(texts(&clean), "nothing sensitive here");
}

#[tokio::test]
async fn a_result_the_module_could_not_check_is_withheld() {
    let error = redact_outcome_with(
        Ok(tinytools::ToolResult::success(format!(
            "token {LONG_KEY} in transit"
        ))),
        &keys(),
        cannot_answer,
    )
    .await
    .unwrap_err();
    assert_eq!(error.to_string(), WITHHELD);
    assert!(!error.to_string().contains(LONG_KEY));
}

#[tokio::test]
async fn an_error_text_is_redacted_or_withheld_never_passed_on() {
    let redacted = redact_outcome_with(
        Err(anyhow::anyhow!("bad credential {LONG_KEY}")),
        &keys(),
        replace_longest_first,
    )
    .await
    .unwrap_err();
    assert_eq!(redacted.to_string(), "bad credential [REDACTED]");
    let withheld = redact_outcome_with(
        Err(anyhow::anyhow!("bad credential {LONG_KEY}")),
        &keys(),
        cannot_answer,
    )
    .await
    .unwrap_err();
    assert_eq!(withheld.to_string(), WITHHELD);
    // A short, key-free error is returned unchanged without asking the module.
    let short = redact_outcome_with(Err(anyhow::anyhow!("nope")), &keys(), cannot_answer)
        .await
        .unwrap_err();
    assert_eq!(short.to_string(), "nope");
}

/// With no module serving (a unit-test process), the production entry points
/// refuse to pass a result that mentions a configured key on, and let one that
/// does not through.
#[tokio::test]
async fn without_a_serving_module_the_production_entry_points_withhold() {
    let config = config_with_overlapping_secrets();
    let outcome = redact_composio_outcome(
        &config,
        Ok(tinytools::ToolResult::success(format!("key {LONG_KEY}"))),
    )
    .await
    .unwrap_err();
    assert_eq!(outcome.to_string(), WITHHELD);
    let rendered = format!("[composio-direct] failed: invalid credential {LONG_KEY}");
    let reported = redact_and_report(&config, "test_op", &rendered).await;
    assert_eq!(reported, WITHHELD);
    assert!(!reported.contains(LONG_KEY) && !reported.contains(SHORT_KEY));
    let unrelated = redact_composio_outcome(
        &config,
        Ok(tinytools::ToolResult::success("a result without any key")),
    )
    .await
    .unwrap();
    assert_eq!(texts(&unrelated), "a result without any key");
}

/// A call with no configured key is not checked: there is nothing exact to remove.
#[tokio::test]
async fn an_unconfigured_call_passes_through_untouched() {
    let mut config = Config::default();
    let tmp = tempfile::tempdir().unwrap();
    config.config_path = tmp.path().join("config.toml");
    let outcome = redact_composio_outcome(&config, Ok(tinytools::ToolResult::success("plain")))
        .await
        .unwrap();
    assert_eq!(texts(&outcome), "plain");
}
