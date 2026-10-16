//! `TinySecurityPolicy`: the single seam that maps the tinysecurity module's
//! [`Verdict`] onto the harness [`PolicyDecision`] (issue #7328, phase 6).
//!
//! The module decides; this adapter only translates. It is a
//! [`ToolCallPolicy`] so the harness `ToolPolicyGate` can sit before every
//! effect-capable middleware. A module fault never allows: the decider
//! returns `Err` and the call is denied with no retry.

use std::sync::Arc;

use async_trait::async_trait;
use tinyagents_harness::context::RunContext;
use tinyagents_harness::middleware::{PolicyDecision, ToolCallPolicy};
use tinyinference_llm::tool::ToolCall as TaToolCall;
use tinysecurity_bus::{DenialReason, Verdict};

use crate::agent::tinyagents::host::OpenHumanRunContext;

/// Where a verdict for one tool call comes from. Production asks the
/// TinySecurity module over its bus contract ([`super::tinysecurity_module`]);
/// the seam exists so the mapping and precedence below are testable without a
/// loaded module. `Err` is a module fault: it never allows and is never retried.
#[async_trait]
pub(crate) trait VerdictSource: Send + Sync {
    async fn verdict(
        &self,
        ctx: &RunContext<OpenHumanRunContext>,
        call: &TaToolCall,
    ) -> Result<Verdict, String>;
}

/// Stable, payload-free text for a denial category.
fn reason_text(reason: &DenialReason) -> &'static str {
    match reason {
        DenialReason::Floor => "blocked by the security floor",
        DenialReason::UnsupportedEffect => "unsupported effect or unknown tool",
        DenialReason::ApprovalUnavailable => "human approval is required",
        DenialReason::Access => "insufficient access",
        DenialReason::PathScope => "path is outside the allowed scope",
        DenialReason::PathResolution => "path could not be securely resolved",
        DenialReason::Command => "command is not safely supported",
        DenialReason::InvalidContext => "missing verified caller identity",
        DenialReason::Content => "content blocked by the security scanner",
    }
}

/// Map a module verdict to the harness decision. `RequireApproval` carries the
/// persisted request id in its reason so the approval resolver can bind to it.
///
/// `Deny { ApprovalUnavailable }` is the module's plain `Evaluate` answer for a
/// host tool whose only remaining requirement is a human ("the weakest answer":
/// every concrete check passed). It maps to `RequireApproval`, never to `Allow`;
/// the gate fails closed when no approval resolver is attached, and the
/// resolver itself denies when no human can be asked. Every other denial stays
/// a denial.
pub(crate) fn verdict_to_policy_decision(verdict: &Verdict) -> PolicyDecision {
    match verdict {
        Verdict::Allow => PolicyDecision::Allow,
        Verdict::Deny {
            reason: DenialReason::ApprovalUnavailable,
        } => PolicyDecision::RequireApproval {
            reason: reason_text(&DenialReason::ApprovalUnavailable).to_string(),
        },
        Verdict::Deny { reason } => PolicyDecision::Deny {
            reason: reason_text(reason).to_string(),
        },
        Verdict::RequireApproval { request_id } => PolicyDecision::RequireApproval {
            reason: format!("approval required (request {request_id})"),
        },
    }
}

/// Fold several decisions; deny dominates, then require-approval, then allow.
pub(crate) fn combine_decisions(decisions: Vec<PolicyDecision>) -> PolicyDecision {
    let mut out = PolicyDecision::Allow;
    for d in decisions {
        match (&out, &d) {
            (PolicyDecision::Deny { .. }, _) => {}
            (_, PolicyDecision::Deny { .. }) => out = d,
            (PolicyDecision::RequireApproval { .. }, _) => {}
            (_, PolicyDecision::RequireApproval { .. }) => out = d,
            _ => {}
        }
    }
    out
}

/// The one security policy every agent path registers. The module decides; this
/// translates. Several sources fold with [`combine_decisions`]: a fault from any
/// source is a denial, so a source can only narrow the answer.
pub(crate) struct TinySecurityPolicy {
    sources: Vec<Arc<dyn VerdictSource>>,
}

impl TinySecurityPolicy {
    pub(crate) fn new(source: Arc<dyn VerdictSource>) -> Self {
        Self::with_sources(vec![source])
    }

    pub(crate) fn with_sources(sources: Vec<Arc<dyn VerdictSource>>) -> Self {
        Self { sources }
    }
}

#[async_trait]
impl ToolCallPolicy<OpenHumanRunContext> for TinySecurityPolicy {
    fn name(&self) -> &str {
        "tinysecurity"
    }

    async fn check(
        &self,
        ctx: &RunContext<OpenHumanRunContext>,
        call: &TaToolCall,
    ) -> PolicyDecision {
        let mut decisions = Vec::with_capacity(self.sources.len());
        for source in &self.sources {
            match source.verdict(ctx, call).await {
                Ok(verdict) => {
                    tracing::debug!(tool = %call.name, "[tinyagents::mw] tinysecurity verdict mapped");
                    decisions.push(verdict_to_policy_decision(&verdict));
                }
                // A fault never allows and is never retried: the call is denied
                // here, without asking the remaining sources.
                Err(_fault) => {
                    tracing::warn!(tool = %call.name, "[tinyagents::mw] tinysecurity module fault; denying");
                    return PolicyDecision::Deny {
                        reason: "security module unavailable".to_string(),
                    };
                }
            }
        }
        combine_decisions(decisions)
    }
}

#[cfg(test)]
#[path = "tinysecurity_tests.rs"]
mod tests;
