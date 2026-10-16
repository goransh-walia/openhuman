use super::*;

/// A refused approval must reach the model as a refusal it can read. Passed on
/// as the gate's own `[policy-denied] User denied ...` text it carried a policy
/// marker, which pauses the turn before the model can reply, and named who refused
/// and how.
#[test]
fn a_refused_approval_is_text_the_model_can_read() {
    for gate_reason in [
        "[policy-denied] User denied 'write_file' execution.",
        "[policy-denied] Approval for 'write_file' timed out after 600s.",
    ] {
        let reason = refused_approval_text("write_file", gate_reason);
        // A policy-class failure pauses the turn before the model can reply.
        assert_eq!(
            tinytools::status::classify(&reason, false).class,
            tinytools::status::ToolFailureClass::Unknown,
            "{reason}"
        );
        assert!(reason.contains("must not be performed"), "{reason}");
        assert!(reason.contains("another way"), "{reason}");
        let lower = reason.to_lowercase();
        assert!(
            !lower.contains("user denied")
                && !lower.contains("timed out")
                && !lower.contains("human"),
            "the text must not say who refused or how: {reason}"
        );
    }
}

/// A prompt that expired unanswered is reported as that, not as a refusal and not
/// as nothing, so the agent tells the user the action is waiting on their
/// approval. It still carries no policy marker, so the turn is not paused before
/// the model can reply, and it names no human.
#[test]
fn an_unanswered_approval_tells_the_model_it_was_not_answered() {
    // The exact reason the gate's TTL path produces.
    let gate_reason = format!(
        "[policy-denied] Approval for 'media_generate_image' timed out after 180s: {}, so it \
         was not run.",
        crate::security::approval::APPROVAL_UNANSWERED_PHRASE
    );
    let reason = refused_approval_text("media_generate_image", &gate_reason);
    assert!(reason.contains("not answered"), "{reason}");
    assert!(reason.contains("ask again"), "{reason}");
    assert!(reason.contains("another way"), "{reason}");
    assert!(reason.contains("'media_generate_image'"), "{reason}");
    assert_eq!(
        tinytools::status::classify(&reason, false).class,
        tinytools::status::ToolFailureClass::Unknown,
        "{reason}"
    );
    let lower = reason.to_lowercase();
    assert!(
        !lower.contains("user denied") && !lower.contains("declined") && !lower.contains("human"),
        "{reason}"
    );
}

#[test]
fn the_desktop_exception_is_off_when_the_tool_is_unknown() {
    let middleware = ApprovalSecurityMiddleware::new(Vec::new());
    assert!(middleware.resolve_tool("missing").is_none());
    let disabled = futures::executor::block_on(middleware.desktop_approvals_disabled(
        &DispatchCtx::ambient(),
        "missing",
    ));
    assert!(!disabled);
}
