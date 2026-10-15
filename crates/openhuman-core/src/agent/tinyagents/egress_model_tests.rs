use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use crate::security::live_policy::test_privacy_scope;
use tinyinference_llm::message::Message;

fn assessed(level: IdentificationRisk, categories: &[&str]) -> Option<Assessment> {
    Some(Assessment {
        level,
        categories: categories.iter().map(|c| c.to_string()).collect(),
    })
}

#[test]
fn non_sensitive_modes_never_refuse() {
    for mode in [PrivacyMode::Standard, PrivacyMode::LocalOnly] {
        assert!(refusal_with(
            mode,
            "openai",
            "gpt",
            assessed(IdentificationRisk::High, &["credential_or_identifier"])
        )
        .is_none());
        assert!(refusal_with(mode, "openai", "gpt", None).is_none());
    }
}

#[test]
fn sensitive_permits_a_request_assessed_clean() {
    assert!(refusal_with(
        PrivacyMode::Sensitive,
        "openai",
        "gpt",
        assessed(IdentificationRisk::None, &[])
    )
    .is_none());
}

#[test]
fn sensitive_refuses_identifying_unassessed_and_missing_assessments() {
    for assessment in [
        assessed(IdentificationRisk::High, &["credential_or_identifier"]),
        assessed(IdentificationRisk::Unknown, &[]),
        None,
    ] {
        let message =
            refusal_with(PrivacyMode::Sensitive, "openai", "gpt", assessment).expect("refused");
        assert!(message.contains("Sensitive privacy mode"), "{message}");
    }
}

struct Counting {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl ChatModel<()> for Counting {
    async fn invoke(
        &self,
        _state: &(),
        _request: ModelRequest,
    ) -> tinyinference_llm::Result<ModelResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ModelResponse::assistant("ok"))
    }
}

fn request() -> ModelRequest {
    ModelRequest::new(vec![Message::user("hello")])
}

fn route(local: bool) -> (Arc<dyn ChatModel<()>>, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let inner: Arc<dyn ChatModel<()>> = Arc::new(Counting {
        calls: calls.clone(),
    });
    (
        EgressAssessedModel::for_route(inner, "openai", "gpt", local),
        calls,
    )
}

#[tokio::test]
async fn an_external_route_passes_straight_through_outside_sensitive_mode() {
    let _mode = test_privacy_scope(PrivacyMode::Standard);
    let (model, calls) = route(false);
    model.invoke(&(), request()).await.expect("allowed");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn under_sensitive_an_unassessable_request_is_refused_before_the_route_is_called() {
    // No module is serving in a unit test, so the assessment is "unknown" and
    // Sensitive refuses it: the safe answer when the module cannot say.
    let _mode = test_privacy_scope(PrivacyMode::Sensitive);
    let (model, calls) = route(false);
    let error = model.invoke(&(), request()).await.err().expect("refused");
    assert!(
        error.to_string().contains("Sensitive privacy mode"),
        "{error}"
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "the route must not be called"
    );
    let streamed = model.stream(&(), request()).await.err().expect("refused");
    assert!(
        streamed.to_string().contains("Sensitive privacy mode"),
        "{streamed}"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_route_that_stays_on_the_device_is_never_assessed() {
    let _mode = test_privacy_scope(PrivacyMode::Sensitive);
    let (model, calls) = route(true);
    model
        .invoke(&(), request())
        .await
        .expect("local route is not assessed");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// A local primary with an external fallback: each route is wrapped on its own,
/// so the fallback hop is assessed even though the primary never leaves.
#[tokio::test]
async fn a_local_primary_does_not_exempt_an_external_fallback_route() {
    let _mode = test_privacy_scope(PrivacyMode::Sensitive);
    let (primary, primary_calls) = route(true);
    let (fallback, fallback_calls) = route(false);
    primary
        .invoke(&(), request())
        .await
        .expect("primary is local");
    assert!(fallback.invoke(&(), request()).await.is_err());
    assert_eq!(primary_calls.load(Ordering::SeqCst), 1);
    assert_eq!(fallback_calls.load(Ordering::SeqCst), 0);
}
