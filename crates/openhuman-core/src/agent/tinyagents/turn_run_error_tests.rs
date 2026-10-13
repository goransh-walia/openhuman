use super::*;

#[tokio::test]
async fn a_timeout_is_not_reported_as_a_provider_error_left_in_the_slot() {
    // #6724: a provider error from an earlier attempt must not be re-surfaced
    // as the cause of a run that timed out or was cancelled.
    for error in [
        tinyagents_harness::TinyAgentsError::CallTimeout("model call".to_string()),
        tinyagents_harness::TinyAgentsError::Timeout("run".to_string()),
        tinyagents_harness::TinyAgentsError::Cancelled,
    ] {
        let slot: ModelErrorSlot = std::sync::Arc::new(std::sync::Mutex::new(Some(
            tinyinference_llm::Error::Model(
                "OpenHuman returned HTTP 503: upstream unavailable".to_string(),
            ),
        )));
        let mapped = map_turn_run_error(error, "m", 10, &slot, None).await;
        assert!(
            !mapped.to_string().contains("503"),
            "stale provider error re-surfaced: {mapped}"
        );
    }
}
