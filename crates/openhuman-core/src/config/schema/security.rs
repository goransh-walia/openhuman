//! The single `[security]` table: audit, approval and the overlay that carries
//! the sandbox, autonomy and privacy policy.
//!
//! Policy settings historically lived in three top-level tables (`[autonomy]`,
//! `[sandbox]`, `[privacy]`). `[security.autonomy]`, `[security.sandbox]` and
//! `[security.privacy]` are the canonical spelling; the pre-decode rewrite in
//! [`crate::config::migrations::security_policy`] overlays them onto the legacy
//! tables key by key (the `[security.*]` value wins) before the typed decode, so
//! consumers keep reading `Config::{autonomy, sandbox, privacy}`. The policy
//! stays off unless `autonomy.enabled = true`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `[security]`: everything that decides what an agent may do.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(default)]
pub struct SecurityConfig {
    /// `[security.audit]`: append-only audit log of policy decisions.
    pub audit: AuditConfig,
    /// `[security.approval]`: interactive approval timing.
    pub approval: ApprovalSettings,
}

/// `[security.approval]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ApprovalSettings {
    /// Seconds an interactive approval stays parked before it expires as
    /// denied. `None` keeps the built-in ten minutes.
    pub ttl_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SandboxConfig {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub backend: SandboxBackend,
    #[serde(default)]
    pub firejail_args: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: None,
            backend: SandboxBackend::Auto,
            firejail_args: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SandboxBackend {
    #[default]
    Auto,
    Landlock,
    Firejail,
    Bubblewrap,
    Docker,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
#[derive(Default)]
pub struct ResourceLimitsConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct AuditConfig {
    #[serde(default = "default_audit_enabled")]
    pub enabled: bool,
    #[serde(default = "default_audit_log_path")]
    pub log_path: String,
    #[serde(default = "default_audit_max_size_mb")]
    pub max_size_mb: u32,
}

fn default_audit_enabled() -> bool {
    true
}

fn default_audit_log_path() -> String {
    "audit.log".to_string()
}

fn default_audit_max_size_mb() -> u32 {
    100
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: default_audit_enabled(),
            log_path: default_audit_log_path(),
            max_size_mb: default_audit_max_size_mb(),
        }
    }
}

#[cfg(test)]
#[path = "security_tests.rs"]
mod tests;
