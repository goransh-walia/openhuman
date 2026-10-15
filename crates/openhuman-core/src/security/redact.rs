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
/// callers that persist or show the result: audit rows, halt summaries and
/// failure lines. Waits up to [`TEXT_TIMEOUT`] for the module.
#[must_use]
pub fn text(text: &str) -> String {
    redact(Kind::Text, text)
}

/// Longest [`text`] waits for the module; large artifact bodies need more than
/// a log line does.
const TEXT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

enum Kind {
    Origin,
    Host,
    Text,
}

#[cfg(feature = "security-module")]
fn redact(kind: Kind, input: &str) -> String {
    use crate::modules::security::text::{redact_for_log, RedactMode};
    let (mode, timeout) = match kind {
        Kind::Origin => (RedactMode::EndpointOrigin, None),
        Kind::Host => (RedactMode::EndpointHost, None),
        Kind::Text => (RedactMode::Text, Some(TEXT_TIMEOUT)),
    };
    redact_for_log(mode, input, timeout)
}

#[cfg(not(feature = "security-module"))]
fn redact(_kind: Kind, _input: &str) -> String {
    crate::core::log_redaction::SUPPRESSED.to_string()
}

#[cfg(test)]
#[path = "redact_tests.rs"]
mod tests;
