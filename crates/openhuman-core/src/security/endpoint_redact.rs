//! Endpoint redaction for log fields, answered by the TinySecurity module.
//!
//! The module owns what an endpoint may reveal (`EndpointOrigin`,
//! `EndpointHost`): credentials, paths, queries and fragments never survive.
//! These helpers exist because `tracing` fields are synchronous; they go
//! through the shared sync seam and answer the suppression marker, never the
//! original URL, when the module cannot (not armed yet, faulted, timed out, or
//! built without `security-module`).

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

enum Kind {
    Origin,
    Host,
}

#[cfg(feature = "security-module")]
fn redact(kind: Kind, url: &str) -> String {
    use crate::modules::security::text::{redact_for_log, RedactMode};
    redact_for_log(
        match kind {
            Kind::Origin => RedactMode::EndpointOrigin,
            Kind::Host => RedactMode::EndpointHost,
        },
        url,
    )
}

#[cfg(not(feature = "security-module"))]
fn redact(_kind: Kind, _url: &str) -> String {
    crate::core::log_redaction::SUPPRESSED.to_string()
}

#[cfg(test)]
#[path = "endpoint_redact_tests.rs"]
mod tests;
