use super::*;

#[tokio::test]
async fn credential_guard_allows_unchanged_text() {
    let result = credentials_removed("ordinary flow memory", async {
        Ok::<_, &'static str>("ordinary flow memory".to_string())
    })
    .await;

    assert_eq!(result, Ok(false));
}

#[tokio::test]
async fn credential_guard_rejects_text_changed_by_redaction() {
    let result = credentials_removed("token=secret-value", async {
        Ok::<_, &'static str>("token=[REDACTED]".to_string())
    })
    .await;

    assert_eq!(result, Ok(true));
}

#[tokio::test]
async fn credential_guard_propagates_redaction_failure() {
    let result = credentials_removed("maybe secret", async {
        Err::<String, _>("TinySecurity unavailable")
    })
    .await;

    assert_eq!(result, Err("TinySecurity unavailable"));
}
