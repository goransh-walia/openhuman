//! `wrap_model`: the privacy epic's payload-time egress check for an inference
//! call that leaves the device.
//!
//! The factory discloses an inference destination when it builds the model, long
//! before a request exists. What the request contains is only known here, so
//! this wrap fills the descriptor's identification risk from the real outgoing
//! messages (`security::egress::assess`, answered by the TinySecurity module)
//! and applies the live privacy mode to it. Only `Sensitive` depends on the
//! assessment: a clean request goes out, one the redactor would change (a
//! credential or identifier) or one the module cannot assess is refused with a
//! non-sensitive message. Under any other mode the wrap passes straight through
//! without a module call.

use async_trait::async_trait;
use tinyagents_harness::context::RunContext;
use tinyagents_harness::middleware::{MiddlewareModelOutcome, ModelHandler, ModelMiddleware};
use tinyinference_llm::model::ModelRequest;

use crate::config::PrivacyMode;
use crate::security::egress::{
    assess::{assess_for_mode, Assessment},
    egress_blocks_message, EgressDescriptor, IdentificationRisk,
};
use crate::security::live_policy::current_privacy_mode;

/// Largest serialized request the host will hand to the module for assessment.
/// A larger request is treated as not assessable, which Sensitive refuses,
/// rather than sent to the module whole.
const MAX_ASSESSED_BYTES: usize = 8 * 1024 * 1024;

pub(crate) struct EgressAssessMiddleware {
    provider: String,
    model: String,
}

impl EgressAssessMiddleware {
    pub(crate) fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
        }
    }
}

/// The refusal for sending a request with the given `assessment` to
/// `provider`/`model` under `mode`, or `None` when the call may proceed. Pure, so
/// the decision is testable without a module; `None` assessment means the module
/// gave no answer.
pub(crate) fn refusal_with(
    mode: PrivacyMode,
    provider: &str,
    model: &str,
    assessment: Option<Assessment>,
) -> Option<String> {
    if mode != PrivacyMode::Sensitive {
        return None;
    }
    let mut descriptor = EgressDescriptor::inference(provider, model, true);
    if let Some(assessment) = assessment.filter(|a| a.level != IdentificationRisk::Unknown) {
        descriptor = descriptor.with_risk(assessment.level, assessment.categories);
    }
    egress_blocks_message(mode, &descriptor)
}

#[async_trait]
impl ModelMiddleware<(), crate::agent::tinyagents::host::OpenHumanRunContext>
    for EgressAssessMiddleware
{
    fn name(&self) -> &str {
        "openhuman.egress_assess"
    }

    async fn wrap_model(
        &self,
        ctx: &mut RunContext<crate::agent::tinyagents::host::OpenHumanRunContext>,
        state: &(),
        request: ModelRequest,
        next: ModelHandler<'_, (), crate::agent::tinyagents::host::OpenHumanRunContext>,
    ) -> tinyagents_harness::Result<MiddlewareModelOutcome> {
        let mode = current_privacy_mode();
        if mode == PrivacyMode::Sensitive {
            let payload = serde_json::to_string(&request.messages)
                .ok()
                .filter(|text| text.len() <= MAX_ASSESSED_BYTES);
            let assessment = payload.as_deref().map(|text| assess_for_mode(mode, text));
            if let Some(message) = refusal_with(mode, &self.provider, &self.model, assessment) {
                tracing::warn!(
                    provider = %self.provider,
                    model = %self.model,
                    "[privacy][egress-assess] inference request refused under Sensitive mode"
                );
                return Err(tinyagents_harness::TinyAgentsError::Middleware(message));
            }
        }
        next.run(ctx, state, request).await
    }
}

#[cfg(test)]
#[path = "egress_assess_tests.rs"]
mod tests;
