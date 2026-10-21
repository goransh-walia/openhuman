use super::*;
use crate::agent::tinyagents::host::OpenHumanRunContext;
use async_trait::async_trait;
use tinyagents_harness::context::{RunConfig, RunContext};
use tinyinference_llm::tool::ToolCall as TaToolCall;
use tinytools::{PermissionLevel, Tool, ToolResult};

static AUDIT_TEST_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();

struct ResetAuditFailureLatch;

impl Drop for ResetAuditFailureLatch {
    fn drop(&mut self) {
        REQUIRED_AUDIT_FAILURE.store(false, Ordering::SeqCst);
    }
}

struct ExternalAuditTool;

#[async_trait]
impl Tool for ExternalAuditTool {
    fn name(&self) -> &str {
        "send_external"
    }

    fn description(&self) -> &str {
        "test external effect"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({ "type": "object" })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Write
    }

    fn external_effect(&self) -> bool {
        true
    }

    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        unreachable!("the audit admission test does not execute the tool")
    }
}

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
    let disabled = futures::executor::block_on(
        middleware.desktop_approvals_disabled(&DispatchCtx::ambient(), "missing"),
    );
    assert!(!disabled);
}

#[test]
fn a_latched_audit_failure_routes_external_tools_even_when_desktop_approvals_are_off() {
    assert!(ApprovalSecurityMiddleware::requires_resolver(
        true, true, true,
    ));
    assert!(!ApprovalSecurityMiddleware::requires_resolver(
        true, false, true,
    ));
    assert!(!ApprovalSecurityMiddleware::requires_resolver(
        false, true, false,
    ));
}

#[tokio::test]
async fn an_allowed_external_effect_gets_a_terminal_audit_ticket() {
    let _lock = AUDIT_TEST_LOCK
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    let _reset_latch = ResetAuditFailureLatch;
    let resolver = ApprovalSecurityMiddleware::new(vec![Arc::new(vec![
        Box::new(ExternalAuditTool) as Box<dyn Tool>,
    ])]);
    let ctx = RunContext::new(RunConfig::new("ordinary-audit"), OpenHumanRunContext::new());
    let call = TaToolCall {
        id: "call-ordinary".into(),
        name: "send_external".into(),
        arguments: serde_json::json!({"payload": "sensitive"}),
        invalid: None,
    };

    let admission = resolver.resolve_allowed(&ctx, &call).await;
    match admission {
        ApprovalResolution::Allow {
            ticket: Some(ticket),
        } => {
            assert!(!ticket.is_empty());
        }
        other => panic!("an allowed external effect needs an audit ticket: {other:?}"),
    }
}

#[tokio::test]
async fn an_external_audit_failure_keeps_the_result_and_denies_later_admissions() {
    let _lock = AUDIT_TEST_LOCK
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    let _reset_latch = ResetAuditFailureLatch;
    let resolver = ApprovalSecurityMiddleware::new(vec![Arc::new(vec![
        Box::new(ExternalAuditTool) as Box<dyn Tool>,
    ])]);
    let ctx = RunContext::new(
        RunConfig::new("ordinary-audit-failure"),
        OpenHumanRunContext::new(),
    );
    let call = TaToolCall {
        id: "call-audit-failure".into(),
        name: "send_external".into(),
        arguments: serde_json::json!({"payload": "sensitive"}),
        invalid: None,
    };
    let ApprovalResolution::Allow {
        ticket: Some(ticket),
    } = resolver.resolve_allowed(&ctx, &call).await
    else {
        panic!("the first external call should be admitted before its terminal audit")
    };
    let result = TaToolResult::success("sensitive returned content");
    let recorded = Arc::new(std::sync::Mutex::new(None));
    let observed = recorded.clone();
    let writes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let first_write = writes.clone();
    resolver.record_terminal_with(
        None,
        &ticket,
        ApprovalExecutionOutcome::Result(&result),
        move |event| {
            first_write.fetch_add(1, Ordering::SeqCst);
            *observed.lock().unwrap() = Some(event);
            Err("audit stream unavailable".into())
        },
    );

    assert!(
        !result.is_error,
        "an audit failure cannot rewrite a completed result"
    );
    assert!(REQUIRED_AUDIT_FAILURE.load(Ordering::SeqCst));
    let event = recorded
        .lock()
        .unwrap()
        .take()
        .expect("audit event attempted");
    assert_eq!(
        event.kind,
        crate::security::audit_record::HostAuditKind::Execution
    );
    assert_eq!(event.attribution, "tool:send_external");
    let serialized = event.arguments.to_string();
    assert!(serialized.contains("success"));
    assert!(!serialized.contains("sensitive"));

    let another_resolver =
        ApprovalSecurityMiddleware::new(vec![Arc::new(vec![
            Box::new(ExternalAuditTool) as Box<dyn Tool>
        ])]);
    assert!(matches!(
        another_resolver.resolve_allowed(&ctx, &call).await,
        ApprovalResolution::Deny { .. }
    ));

    let repeated_write = writes.clone();
    resolver.record_terminal_with(
        None,
        &ticket,
        ApprovalExecutionOutcome::Result(&result),
        move |_| {
            repeated_write.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    );
    assert_eq!(
        writes.load(Ordering::SeqCst),
        1,
        "a ticket records only once"
    );
}

#[test]
fn a_required_approval_is_not_allowed_to_run_without_a_gate() {
    let resolution = ApprovalSecurityMiddleware::resolve_without_gate("composio_execute");
    match resolution {
        ApprovalResolution::Deny { reason } => {
            assert!(reason.contains("This action was refused"), "{reason}");
            assert!(reason.contains("must not be performed"), "{reason}");
            assert!(reason.contains("do not retry"), "{reason}");
            assert!(reason.contains("another way"), "{reason}");
            assert!(!reason.to_lowercase().contains("human"), "{reason}");
        }
        ApprovalResolution::Allow { .. } => {
            panic!("a missing human approval gate must not authorize an external effect")
        }
    }
}
