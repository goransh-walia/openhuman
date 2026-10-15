//! The host's one redaction entry point for synchronous callers, answered by
//! the TinySecurity module.
//!
//! The module owns what text and endpoints may reveal (`Text`, `EndpointOrigin`,
//! `EndpointHost`): credentials, paths, queries and fragments never survive.
//! These helpers exist because `tracing` fields, durable audit rows and
//! artifact writers are synchronous; they go through the shared sync seam and
//! answer the suppression marker, never the original text, when the module
//! cannot (not armed yet, faulted, timed out, or built without
//! `security-module`). Async callers use `modules::security::text` directly.

/// `scheme://host[:port]` of `url`, without credentials, path, query or fragment.
#[must_use]
pub fn endpoint_origin(url: &str) -> String {
    redact(Kind::Origin, url)
}

/// Hostname of `url` only (no scheme, port, credentials or path).
#[must_use]
pub fn endpoint_host(url: &str) -> String {
    redact(Kind::Host, url)
}

/// Secrets and PII removed from free `text` (the module's `Text` mode), for
/// callers that persist or show the result: audit rows, halt summaries,
/// failure lines and artifact bodies. Waits up to [`TEXT_TIMEOUT`] for the
/// module.
///
/// This text is data the user still needs (an artifact the model re-reads, an
/// audit row), so unlike an endpoint field it is not blanked when the module
/// cannot answer. The fixed boot floor (`core::log_redaction`, ADR 7) is
/// applied instead, line by line so a large body is not suppressed whole; the
/// original text is never returned unfiltered.
#[must_use]
pub fn text(text: &str) -> String {
    answer_or_floor(module_text(text), text)
}

/// The module's `answer` when it gave one, otherwise the floor over `input`.
fn answer_or_floor(answer: Option<String>, input: &str) -> String {
    match answer {
        Some(redacted) => redacted,
        None => crate::core::log_redaction::floor_scrub_text(input),
    }
}

#[cfg(feature = "security-module")]
fn module_text(input: &str) -> Option<String> {
    use crate::modules::security::text::{try_redact, RedactMode};
    match try_redact(RedactMode::Text, input, Some(TEXT_TIMEOUT)) {
        Ok(redacted) => Some(redacted),
        Err(error) => {
            log::debug!("[security][redact] module text pass unavailable, floor applies: {error}");
            None
        }
    }
}

#[cfg(not(feature = "security-module"))]
fn module_text(_input: &str) -> Option<String> {
    None
}

/// Longest [`text`] waits for the module; large artifact bodies need more than
/// a log line does.
const TEXT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

enum Kind {
    Origin,
    Host,
}

#[cfg(feature = "security-module")]
fn redact(kind: Kind, input: &str) -> String {
    use crate::modules::security::text::{redact_for_log, RedactMode};
    let mode = match kind {
        Kind::Origin => RedactMode::EndpointOrigin,
        Kind::Host => RedactMode::EndpointHost,
    };
    redact_for_log(mode, input, None)
}

#[cfg(not(feature = "security-module"))]
fn redact(_kind: Kind, _input: &str) -> String {
    crate::core::log_redaction::SUPPRESSED.to_string()
}

#[cfg(test)]
#[path = "redact_tests.rs"]
mod tests;
