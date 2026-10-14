use super::*;

fn result(text: &str) -> WebChatTaskResult {
    WebChatTaskResult {
        full_response: text.to_string(),
        citations: Vec::new(),
        usage: None,
        workspace_dir: std::path::PathBuf::from("/tmp/turn-guards"),
        timing: None,
    }
}

/// The guarded turn sees the backstop it runs under, so the session turn can
/// hand the harness the deadline it must wind down before.
#[tokio::test]
async fn guarded_turn_sees_the_backstop_deadline() {
    let backstop = Duration::from_secs(900);
    let seen = std::sync::Arc::new(std::sync::Mutex::new(None));
    let seen_in_turn = seen.clone();
    let res = drive_turn_with_deadline(Some(backstop), async move {
        *seen_in_turn.lock().unwrap() = crate::agent::turn_deadline::current();
        Ok(result("answer"))
    })
    .await
    .expect("turn completes");
    assert_eq!(res.full_response, "answer");
    let deadline = seen.lock().unwrap().expect("deadline scoped on the turn");
    assert_eq!(deadline.backstop(), backstop);
    assert_eq!(deadline.wind_down_after(), Duration::from_secs(780));
    assert!(deadline.wind_down_after() < deadline.hard_stop_after());
    assert!(deadline.hard_stop_after() < backstop);
}

#[tokio::test]
async fn disabled_backstop_scopes_no_deadline() {
    let res = drive_turn_with_deadline(None, async {
        assert!(crate::agent::turn_deadline::current().is_none());
        Ok(result("ok"))
    })
    .await;
    assert!(res.is_ok());
}

/// If the backstop fires anyway, the caller gets the typed `turn_timeout`
/// error, which is classified to user-facing copy, not silence.
#[tokio::test(start_paused = true)]
async fn backstop_firing_yields_a_classified_turn_timeout_error() {
    let err = drive_turn_with_deadline(Some(Duration::from_secs(900)), async {
        std::future::pending::<()>().await;
        Ok(result("never"))
    })
    .await
    .expect_err("backstop fires");
    assert!(super::super::super::web_errors::is_outer_backstop_timeout(
        &err
    ));
    let classified = super::super::super::web_errors::classify_inference_error(&err);
    assert_eq!(classified.error_type, "turn_timeout");
    assert!(!classified.message.trim().is_empty());
}

/// A Stop fires the turn's own session cancellation and lets the turn finish,
/// so the session runtime can persist the stopped turn's rounds (#7715). The
/// turn's result is still discarded: the caller unwinds as a cancel.
#[tokio::test]
async fn stop_fires_the_session_cancellation_and_lets_the_turn_unwind() {
    let cancel = CancellationToken::new();
    let unwound = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let unwound_in_turn = unwound.clone();
    let turn = run_until_stopped(cancel.clone(), async move {
        crate::agent::host_overrides::current_cancellation()
            .cancelled()
            .await;
        unwound_in_turn.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok::<_, String>(result("persisted partial"))
    });
    let stop = async {
        tokio::task::yield_now().await;
        cancel.cancel();
        // A double Stop is a no-op, not a second unwind.
        cancel.cancel();
    };
    let (res, ()) = tokio::join!(turn, stop);
    assert!(res.is_none(), "a stopped turn reports a cancel");
    assert!(
        unwound.load(std::sync::atomic::Ordering::SeqCst),
        "the turn observed its session cancellation and ran to completion"
    );
}

/// A turn that ignores its session cancellation is dropped once the grace
/// runs out, so a Stop can never hang on a stuck turn.
#[tokio::test(start_paused = true)]
async fn stop_drops_a_turn_that_ignores_cancellation_after_the_grace() {
    let cancel = CancellationToken::new();
    cancel.cancel();
    let started = tokio::time::Instant::now();
    let res = run_until_stopped(cancel, std::future::pending::<()>()).await;
    assert!(res.is_none());
    assert!(started.elapsed() >= STOPPED_TURN_PERSIST_GRACE);
}

/// Without a Stop the turn's own result comes back.
#[tokio::test]
async fn unstopped_turn_returns_its_result() {
    let res = run_until_stopped(CancellationToken::new(), async {
        Ok::<_, String>(result("answer"))
    })
    .await;
    assert_eq!(res.expect("not stopped").unwrap().full_response, "answer");
}
