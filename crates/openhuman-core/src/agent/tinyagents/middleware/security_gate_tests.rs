use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;
use tinyagents_harness::context::{RunConfig, RunContext};
use tinyagents_harness::middleware::{
    ApprovalResolution, ApprovalResolver, BoxToolFuture, MiddlewareStack, ToolBaseCall,
};
use tinyinference_llm::tool::ToolCall as TaToolCall;
use tinysecurity_bus::{DenialReason, Verdict};
use tinytools::ToolResult;

use super::super::tinysecurity::{TinySecurityPolicy, VerdictSource};

struct Module {
    answer: Result<Verdict, String>,
    asked: AtomicUsize,
}

#[async_trait]
impl VerdictSource for Module {
    async fn verdict(
        &self,
        _ctx: &RunContext<OpenHumanRunContext>,
        _call: &TaToolCall,
    ) -> Result<Verdict, String> {
        self.asked.fetch_add(1, Ordering::SeqCst);
        self.answer.clone()
    }
}

#[derive(Default)]
struct Human {
    answer: Mutex<Option<ApprovalResolution>>,
    asked: AtomicUsize,
    recorded: Mutex<Vec<String>>,
}

#[async_trait]
impl ApprovalResolver<OpenHumanRunContext> for Human {
    async fn requires_approval(
        &self,
        _ctx: &RunContext<OpenHumanRunContext>,
        _call: &TaToolCall,
    ) -> bool {
        true
    }

    async fn resolve(
        &self,
        _ctx: &RunContext<OpenHumanRunContext>,
        _call: &TaToolCall,
    ) -> ApprovalResolution {
        self.asked.fetch_add(1, Ordering::SeqCst);
        self.answer
            .lock()
            .unwrap()
            .clone()
            .expect("scripted answer")
    }

    fn record(&self, ticket: &str, _result: &ToolResult) {
        self.recorded.lock().unwrap().push(ticket.to_string());
    }
}

struct Tool {
    runs: Arc<AtomicUsize>,
}

impl ToolBaseCall<(), OpenHumanRunContext> for Tool {
    fn call<'a>(
        &'a self,
        _ctx: &'a RunContext<OpenHumanRunContext>,
        _state: &'a (),
        _call: TaToolCall,
    ) -> BoxToolFuture<'a> {
        Box::pin(async move {
            self.runs.fetch_add(1, Ordering::SeqCst);
            Ok(ToolResult::success("ran"))
        })
    }
}

struct Outcome {
    result: ToolResult,
    runs: usize,
    module_asks: usize,
    human_asks: usize,
    recorded: Vec<String>,
}

async fn through_the_gate(
    answer: Result<Verdict, String>,
    human_answer: Option<ApprovalResolution>,
) -> Outcome {
    let module = Arc::new(Module {
        answer,
        asked: AtomicUsize::new(0),
    });
    let human = Arc::new(Human::default());
    *human.answer.lock().unwrap() = human_answer;
    let policy = TinySecurityPolicy::new(module.clone());
    let mut stack: MiddlewareStack<(), OpenHumanRunContext> = MiddlewareStack::new();
    stack.push_tool_middleware(Arc::new(gate(Arc::new(policy), human.clone())));
    let runs = Arc::new(AtomicUsize::new(0));
    let ctx = RunContext::new(RunConfig::new("security-gate"), OpenHumanRunContext::new());
    let call = TaToolCall {
        id: "c1".into(),
        name: "send_email".into(),
        arguments: serde_json::json!({}),
        invalid: None,
    };
    let result = stack
        .run_wrapped_tool(&ctx, &(), call, &Tool { runs: runs.clone() })
        .await
        .unwrap()
        .into_result();
    let recorded = human.recorded.lock().unwrap().clone();
    Outcome {
        result,
        runs: runs.load(Ordering::SeqCst),
        module_asks: module.asked.load(Ordering::SeqCst),
        human_asks: human.asked.load(Ordering::SeqCst),
        recorded,
    }
}

#[tokio::test]
async fn an_allowed_call_runs_without_asking_a_human() {
    let out = through_the_gate(Ok(Verdict::Allow), None).await;
    assert!(!out.result.is_error);
    assert_eq!((out.runs, out.module_asks, out.human_asks), (1, 1, 0));
}

#[tokio::test]
async fn a_denied_call_never_runs_and_never_asks_a_human() {
    let out = through_the_gate(
        Ok(Verdict::Deny {
            reason: DenialReason::Floor,
        }),
        Some(ApprovalResolution::Allow { ticket: None }),
    )
    .await;
    assert!(out.result.is_error);
    assert!(out.result.output().contains("tinysecurity"));
    assert_eq!((out.runs, out.human_asks), (0, 0), "deny beats approval");
}

#[tokio::test]
async fn a_call_the_module_leaves_to_a_human_runs_only_when_approved() {
    let needs_human = || {
        Ok(Verdict::Deny {
            reason: DenialReason::ApprovalUnavailable,
        })
    };
    let approved = through_the_gate(
        needs_human(),
        Some(ApprovalResolution::Allow {
            ticket: Some("t-1".into()),
        }),
    )
    .await;
    assert_eq!((approved.runs, approved.human_asks), (1, 1));
    assert_eq!(approved.recorded, vec!["t-1".to_string()]);

    let refused = through_the_gate(
        needs_human(),
        Some(ApprovalResolution::Deny {
            reason: "user said no".into(),
        }),
    )
    .await;
    assert!(refused.result.is_error);
    assert!(refused.result.output().contains("user said no"));
    assert_eq!(refused.runs, 0);
}

#[tokio::test]
async fn a_required_approval_verdict_asks_the_human() {
    let out = through_the_gate(
        Ok(Verdict::RequireApproval {
            request_id: "req-1".into(),
        }),
        Some(ApprovalResolution::Deny {
            reason: "not now".into(),
        }),
    )
    .await;
    assert_eq!((out.runs, out.human_asks), (0, 1));
}

#[tokio::test]
async fn a_module_fault_denies_once_and_does_not_fall_back_to_a_human() {
    let out = through_the_gate(
        Err("module crashed".into()),
        Some(ApprovalResolution::Allow { ticket: None }),
    )
    .await;
    assert!(out.result.is_error);
    assert_eq!(
        (out.runs, out.module_asks, out.human_asks),
        (0, 1, 0),
        "a fault denies with no retry and no approval path"
    );
}
