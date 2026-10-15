//! [`redact_composio_outcome`]: keep a Composio API key out of what a
//! Composio tool returns to the model.
//!
//! Which keys a call can resolve to is host configuration ([`composio_secrets`]).
//! Removing them is the TinySecurity module's job: every string in the outcome
//! goes to `RedactWith` with this call's keys as exact credentials, so a tenant's
//! own key is removed from that tenant's output without a process-wide
//! reinitialization and without a second redactor here. This file walks the
//! outcome, hands the module its strings and writes the answers back; a result
//! the module could not check is withheld, never passed on.

use std::collections::HashMap;
use std::future::Future;

use serde_json::Value;
use tinytools::{ToolContent, ToolResult};

use crate::config::Config;

const MIN_SECRET_LEN: usize = 8;

/// The Composio keys `config` can resolve to: a host-pinned key, the inline
/// `composio.api_key`, and in unpinned direct mode the stored key.
pub(crate) fn composio_secrets(config: &Config) -> Vec<String> {
    let composio = &config.composio;
    let mut secrets: Vec<String> = Vec::new();
    if let Some(pinned) = composio.host_credential.as_ref() {
        secrets.push(pinned.api_key().to_string());
    }
    if let Some(inline) = composio.api_key.as_deref() {
        secrets.push(inline.trim().to_string());
    }
    if composio.host_credential.is_none()
        && composio.mode.trim() == crate::config::schema::COMPOSIO_MODE_DIRECT
    {
        if let Ok(Some(stored)) = crate::security::credentials::get_composio_api_key(config) {
            secrets.push(stored);
        }
    }
    secrets.retain(|s| s.len() >= MIN_SECRET_LEN);
    secrets.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    secrets.dedup();
    secrets
}

/// Only a string that actually contains one of the call's keys is sent to the
/// module. This decides whether to ask, never what to remove: the module still
/// removes the key (and anything else it recognizes) from every string it is
/// given. Most results never mention the key, so most results cost no module
/// call at all.
fn mentions_a_key(text: &str, secrets: &[String]) -> bool {
    secrets.iter().any(|secret| text.contains(secret.as_str()))
}

/// What a result becomes when the module could not check it.
pub(crate) const WITHHELD: &str = "Composio result withheld: the security module could not check it for credentials. Do not retry; tell the user the result could not be checked.";

/// Every string the outcome carries that could contain a key: text, JSON string
/// values and object keys, follow-up content, the markdown rendering and the
/// metadata.
fn strings_of(result: &ToolResult, secrets: &[String]) -> Vec<String> {
    fn json(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::String(text) => out.push(text.clone()),
            Value::Array(items) => items.iter().for_each(|item| json(item, out)),
            Value::Object(map) => {
                for (key, value) in map {
                    out.push(key.clone());
                    json(value, out);
                }
            }
            _ => {}
        }
    }
    fn content(block: &ToolContent, out: &mut Vec<String>) {
        match block {
            ToolContent::Text { text } => out.push(text.clone()),
            ToolContent::Json { data } => json(data, out),
            _ => {}
        }
    }
    let mut out = Vec::new();
    result.content.iter().for_each(|c| content(c, &mut out));
    result.follow_up.iter().for_each(|c| content(c, &mut out));
    if let Some(markdown) = &result.markdown_formatted {
        out.push(markdown.clone());
    }
    if let Some(metadata) = &result.metadata {
        json(metadata, &mut out);
    }
    out.retain(|text| mentions_a_key(text, secrets));
    out.sort();
    out.dedup();
    out
}

/// Write `answers` (original string to redacted string) back into `result`.
/// A string without an answer is left as it is (it never mentioned a key).
fn apply_answers(result: &mut ToolResult, answers: &HashMap<String, String>) {
    let map = |text: &str| {
        answers
            .get(text)
            .cloned()
            .unwrap_or_else(|| text.to_owned())
    };
    fn json(value: &mut Value, map: &dyn Fn(&str) -> String) {
        match value {
            Value::String(text) => *text = map(text),
            Value::Array(items) => items.iter_mut().for_each(|item| json(item, map)),
            Value::Object(object) => {
                let entries = std::mem::take(object);
                for (key, mut value) in entries {
                    json(&mut value, map);
                    let mut key = map(&key);
                    // Two keys can redact to the same marker; keep both values.
                    if object.contains_key(&key) {
                        let base = key.clone();
                        let mut suffix = 2;
                        loop {
                            key = format!("{base} ({suffix})");
                            if !object.contains_key(&key) {
                                break;
                            }
                            suffix += 1;
                        }
                    }
                    object.insert(key, value);
                }
            }
            _ => {}
        }
    }
    fn content(block: &mut ToolContent, map: &dyn Fn(&str) -> String) {
        match block {
            ToolContent::Text { text } => *text = map(text),
            ToolContent::Json { data } => json(data, map),
            _ => {}
        }
    }
    result.content.iter_mut().for_each(|c| content(c, &map));
    result.follow_up.iter_mut().for_each(|c| content(c, &map));
    if let Some(markdown) = result.markdown_formatted.as_mut() {
        *markdown = map(markdown);
    }
    if let Some(metadata) = result.metadata.as_mut() {
        json(metadata, &map);
    }
}

/// `redactor` receives the strings to check and answers each one, or `Err` when
/// the module could not.
pub(crate) type Answers = Result<HashMap<String, String>, String>;

/// [`redact_composio_outcome`] over an explicit redactor, so the walk is
/// testable without a module. An outcome that mentions a key and that the
/// redactor could not check is withheld.
pub(crate) async fn redact_outcome_with<R, Fut>(
    outcome: anyhow::Result<ToolResult>,
    secrets: &[String],
    redactor: R,
) -> anyhow::Result<ToolResult>
where
    R: Fn(Vec<String>) -> Fut,
    Fut: Future<Output = Answers>,
{
    match outcome {
        Ok(mut result) => {
            let strings = strings_of(&result, secrets);
            if strings.is_empty() {
                return Ok(result);
            }
            match redactor(strings).await {
                Ok(answers) => {
                    apply_answers(&mut result, &answers);
                    Ok(result)
                }
                Err(error) => {
                    tracing::warn!("[composio] result withheld: {error}");
                    Err(anyhow::anyhow!(WITHHELD))
                }
            }
        }
        Err(error) => {
            let rendered = format!("{error:#}");
            if !mentions_a_key(&rendered, secrets) {
                return Err(error);
            }
            match redactor(vec![rendered.clone()]).await {
                Ok(answers) => match answers.get(&rendered) {
                    Some(redacted) if *redacted != rendered => {
                        tracing::debug!("[composio] redacted a credential from a tool error");
                        Err(anyhow::anyhow!(redacted.clone()))
                    }
                    _ => Err(error),
                },
                Err(why) => {
                    tracing::warn!("[composio] error text withheld: {why}");
                    Err(anyhow::anyhow!(WITHHELD))
                }
            }
        }
    }
}

/// The module's answer for each of `strings` with `secrets` as this call's exact
/// credentials.
#[cfg(feature = "security-module")]
async fn module_answers(config: &Config, secrets: &[String], strings: Vec<String>) -> Answers {
    let answers = futures::future::join_all(strings.iter().map(|text| {
        crate::modules::security::text::redact_credentials_with(config, text, secrets)
    }))
    .await;
    strings
        .into_iter()
        .zip(answers)
        .map(|(text, answer)| answer.map(|redacted| (text, redacted)))
        .collect()
}

/// Without the security module nothing can check a string that mentions a key,
/// so every such string is withheld (fail closed).
#[cfg(not(feature = "security-module"))]
async fn module_answers(_config: &Config, _secrets: &[String], _strings: Vec<String>) -> Answers {
    Err("the security module is not built into this host".to_string())
}

/// The module's redaction of one string with `secrets` as exact credentials.
#[cfg(feature = "security-module")]
async fn redact_one(config: &Config, text: &str, secrets: &[String]) -> Result<String, String> {
    crate::modules::security::text::redact_credentials_with(config, text, secrets).await
}

#[cfg(not(feature = "security-module"))]
async fn redact_one(_config: &Config, _text: &str, _secrets: &[String]) -> Result<String, String> {
    Err("the security module is not built into this host".to_string())
}

/// Redact every Composio secret `config` resolves to from `rendered`, report
/// the redacted copy via
/// [`report_composio_op_error`](super::super::ops::report_composio_op_error),
/// and return that same redacted copy for the caller's own error text.
pub(crate) async fn redact_and_report(config: &Config, operation: &str, rendered: &str) -> String {
    let secrets = composio_secrets(config);
    let redacted = if !mentions_a_key(rendered, &secrets) {
        rendered.to_string()
    } else {
        match redact_one(config, rendered, &secrets).await {
            Ok(redacted) => redacted,
            Err(error) => {
                tracing::warn!("[composio] error text withheld: {error}");
                WITHHELD.to_string()
            }
        }
    };
    super::super::ops::report_composio_op_error(operation, &redacted);
    redacted
}

/// Redact every Composio key `config` resolves to from a tool's result or
/// error before it reaches the model.
pub(crate) async fn redact_composio_outcome(
    config: &Config,
    outcome: anyhow::Result<ToolResult>,
) -> anyhow::Result<ToolResult> {
    let secrets = composio_secrets(config);
    if secrets.is_empty() {
        return outcome;
    }
    redact_outcome_with(outcome, &secrets, |strings| {
        module_answers(config, &secrets, strings)
    })
    .await
}
