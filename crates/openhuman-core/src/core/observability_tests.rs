use super::*;
use crate::inference::registry::{
    register_backend_error_classifier, BackendErrorClassification, BackendErrorKind,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(feature = "crash-reporting")]
fn event_with_tags(pairs: &[(&str, &str)]) -> sentry::protocol::Event<'static> {
    let mut event = sentry::protocol::Event::default();
    let mut tags: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for (k, v) in pairs {
        tags.insert((*k).to_string(), (*v).to_string());
    }
    event.tags = tags;
    event
}

#[cfg(feature = "crash-reporting")]
fn event_with_message(msg: &str) -> sentry::protocol::Event<'static> {
    let mut event = sentry::protocol::Event::default();
    event.message = Some(msg.to_string());
    event
}

#[cfg(feature = "crash-reporting")]
fn channel_message_404_event(method: &str) -> sentry::protocol::Event<'static> {
    let mut event = sentry::protocol::Event::default();
    event.tags.insert("domain".into(), "backend_api".into());
    event.tags.insert("failure".into(), "non_2xx".into());
    event.tags.insert("status".into(), "404".into());
    event.tags.insert("method".into(), method.into());
    event.message = Some(
        "PATCH /channels/telegram/messages/1103 failed (404); response_body_len=172".to_string(),
    );
    event
}

fn managed_body(status: &str, code: &str) -> String {
    let body = format!(
        "OpenHuman API error ({status}): {{\"error\":{{\"errorCode\":\"{code}\",\"message\":\"x\"}}}}"
    );
    register_backend_classifier_fixture(
        &body,
        BackendErrorClassification {
            provider_owned: true,
            backend_owned: true,
            reportable: false,
            token: Some("fixture-owned".to_string()),
            kind: Some(BackendErrorKind::Internal),
            malformed_bad_request: false,
        },
    );
    body
}

/// Installs exact-message metadata for core tests of the generic classifier
/// hook. The tests supply ownership and reportability directly; this fixture
/// deliberately contains no hosted token parser or vocabulary.
fn register_backend_classifier_fixture(message: &str, classification: BackendErrorClassification) {
    static FIXTURES: OnceLock<Arc<Mutex<HashMap<String, BackendErrorClassification>>>> =
        OnceLock::new();
    let fixtures = Arc::clone(FIXTURES.get_or_init(|| Arc::new(Mutex::new(HashMap::new()))));
    let classifier_fixtures = Arc::clone(&fixtures);
    register_backend_error_classifier(Arc::new(move |_provider, error| {
        classifier_fixtures
            .lock()
            .expect("classifier fixture lock")
            .get(error)
            .cloned()
            .unwrap_or(BackendErrorClassification {
                provider_owned: false,
                backend_owned: false,
                reportable: true,
                token: None,
                kind: None,
                malformed_bad_request: false,
            })
    }));
    fixtures
        .lock()
        .expect("classifier fixture lock")
        .insert(message.to_string(), classification);
}

fn reportable_fixture(
    kind: BackendErrorKind,
    malformed_bad_request: bool,
) -> BackendErrorClassification {
    BackendErrorClassification {
        provider_owned: true,
        backend_owned: true,
        reportable: true,
        token: Some("fixture-reportable".to_string()),
        kind: Some(kind),
        malformed_bad_request,
    }
}

#[cfg(feature = "crash-reporting")]
fn event_with_tags_and_message(
    pairs: &[(&str, &str)],
    message: &str,
) -> sentry::protocol::Event<'static> {
    let mut event = event_with_tags(pairs);
    event.message = Some(message.to_string());
    event
}

#[cfg(feature = "crash-reporting")]
fn event_with_exception_value(value: &str) -> sentry::protocol::Event<'static> {
    let mut event = sentry::protocol::Event::default();
    event.exception = vec![sentry::protocol::Exception {
        value: Some(value.to_string()),
        ..Default::default()
    }]
    .into();
    event
}

#[path = "observability_crash_filter_integrations_tests.rs"]
mod crash_filter_integrations;
#[path = "observability_crash_filter_messages_tests.rs"]
mod crash_filter_messages;
#[path = "observability_error_classification_core_tests.rs"]
mod error_classification_core;
#[path = "observability_error_classification_network_tests.rs"]
mod error_classification_network;
#[path = "observability_error_classification_provider_tests.rs"]
mod error_classification_provider;
#[path = "observability_error_classification_session_tests.rs"]
mod error_classification_session;
#[path = "observability_error_classification_user_state_tests.rs"]
mod error_classification_user_state;
