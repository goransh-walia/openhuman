//! The privacy epic's payload-time egress check for an inference call that
//! leaves the device, applied to **each route** that does.
//!
//! The factory discloses an inference destination when it builds the model,
//! long before a request exists. What the request contains is only known when a
//! route is called, so this decorator fills the descriptor's identification
//! risk from the real outgoing messages (`security::egress::assess`, answered
//! by the TinySecurity module) and applies the live privacy mode to it. Only
//! `Sensitive` depends on the assessment: a clean request goes out, one the
//! redactor would change (a credential or identifier) or one the module cannot
//! assess is refused with a non-sensitive message. Under any other mode the
//! decorator passes straight through without a module call.
//!
//! It wraps every external route the turn can reach, not the turn's primary
//! alone: a local primary with an external fallback (or an external tier route)
//! would otherwise send its fallback request unassessed. A refused hop fails
//! that hop only, so the harness's fallback traversal may still move on to a
//! route that stays on the device.

use std::sync::Arc;

use async_trait::async_trait;
use tinyinference_llm::model::{
    ChatModel, InputModality, InputSource, ModelProfile, ModelRequest, ModelResponse, ModelStream,
};

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

/// A route whose requests leave the device, assessed before each call.
pub(crate) struct EgressAssessedModel {
    inner: Arc<dyn ChatModel<()>>,
    provider: String,
    model: String,
}

impl EgressAssessedModel {
    pub(crate) fn new(
        inner: Arc<dyn ChatModel<()>>,
        provider: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            inner,
            provider: provider.into(),
            model: model.into(),
        }
    }

    /// Wrap `inner` when `provider` is not a local one, otherwise hand it back
    /// unchanged: a request that never leaves the device has nothing to assess.
    pub(crate) fn for_route(
        inner: Arc<dyn ChatModel<()>>,
        provider: &str,
        model: &str,
    ) -> Arc<dyn ChatModel<()>> {
        if tinyinference_local::profile::is_local_provider_string(provider) {
            inner
        } else {
            Arc::new(Self::new(inner, provider, model))
        }
    }

    fn check(&self, request: &ModelRequest) -> tinyinference_llm::Result<()> {
        let mode = current_privacy_mode();
        if mode != PrivacyMode::Sensitive {
            return Ok(());
        }
        let payload = serde_json::to_string(&request.messages)
            .ok()
            .filter(|text| text.len() <= MAX_ASSESSED_BYTES);
        let assessment = payload.as_deref().map(|text| assess_for_mode(mode, text));
        match refusal_with(mode, &self.provider, &self.model, assessment) {
            Some(message) => {
                tracing::warn!(
                    provider = %self.provider,
                    model = %self.model,
                    "[privacy][egress-assess] inference request refused under Sensitive mode"
                );
                Err(tinyinference_llm::Error::Validation(message))
            }
            None => Ok(()),
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
impl ChatModel<()> for EgressAssessedModel {
    fn profile(&self) -> Option<&ModelProfile> {
        self.inner.profile()
    }

    fn supports_input(&self, modality: InputModality, mime_type: &str, source: InputSource) -> bool {
        self.inner.supports_input(modality, mime_type, source)
    }

    fn cache_identity(&self) -> Option<String> {
        self.inner.cache_identity()
    }

    async fn invoke(&self, state: &(), request: ModelRequest) -> tinyinference_llm::Result<ModelResponse> {
        self.check(&request)?;
        self.inner.invoke(state, request).await
    }

    async fn stream(&self, state: &(), request: ModelRequest) -> tinyinference_llm::Result<ModelStream> {
        self.check(&request)?;
        self.inner.stream(state, request).await
    }
}

#[cfg(test)]
#[path = "egress_model_tests.rs"]
mod tests;
