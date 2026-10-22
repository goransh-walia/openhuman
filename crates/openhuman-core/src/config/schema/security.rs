//! The single `[security]` table: autonomy, sandbox, privacy, approval and
//! automatic approval policy.
//!
//! Policy settings historically lived in three top-level tables (`[autonomy]`,
//! `[sandbox]`, `[privacy]`). The pre-decode rewrite in
//! [`crate::config::migrations::security_policy`] folds them into this table
//! key by key, with `[security.*]` values taking precedence. The policy stays
//! off unless `[security.autonomy].enabled = true`.

use super::{AutonomyConfig, PrivacyConfig};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `[security]`: everything that decides what an agent may do.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(default)]
pub struct SecurityConfig {
    /// `[security.autonomy]`: command and tool policy.
    pub autonomy: AutonomyConfig,
    /// `[security.sandbox]`: sandbox backend preferences.
    pub sandbox: SandboxConfig,
    /// `[security.privacy]`: data-egress posture.
    pub privacy: PrivacyConfig,
    /// `[security.approval]`: interactive approval timing.
    pub approval: ApprovalSettings,
    /// `[security.auto_approve]`: automatic approval beyond the per-tool list.
    pub auto_approve: AutoApproveSettings,
}

/// `[security.auto_approve]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct AutoApproveSettings {
    /// `[security.auto_approve.judge]`: the Jev judge. Off by default; any
    /// judge failure, low confidence or budget exhaustion falls to a human.
    pub judge: JudgeSettings,
}

/// `[security.auto_approve.judge]`. Scores are in thousandths (`900` = 0.9).
/// The credential is never stored here: it is resolved with
/// `resolve_backend_credential` when the module is configured.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct JudgeSettings {
    /// Master switch. Default `false`.
    pub enabled: bool,
    /// Origins that opt in (`operator`, `copilot`, `sub_agent`, `cron`,
    /// `saas`). Empty means no origin is judged, even when `enabled`.
    pub origins: Vec<String>,
    /// Minimum intent and reversibility score.
    pub threshold_millis: u16,
    /// Maximum exfiltration score.
    pub exfiltrate_millis: u16,
    /// Total request deadline in milliseconds.
    pub timeout_ms: u32,
    /// Explicit evaluation endpoint; empty uses the provider default.
    pub endpoint: String,
    /// Requests allowed per budget window.
    pub max_requests: u32,
    /// Budget window in milliseconds.
    pub budget_window_ms: u32,
}

impl Default for JudgeSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            origins: Vec::new(),
            threshold_millis: 900,
            exfiltrate_millis: 100,
            timeout_ms: 5_000,
            endpoint: String::new(),
            max_requests: 30,
            budget_window_ms: 60_000,
        }
    }
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
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: None,
            backend: SandboxBackend::Auto,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SandboxBackend {
    #[default]
    Auto,
    Landlock,
    Docker,
    None,
}

#[cfg(test)]
#[path = "security_tests.rs"]
mod tests;
