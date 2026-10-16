//! A scripted [`ApprovalEngine`] for the tests of what the gate does with the
//! engine's answers (parking, routing, events, expiry handling) without loading
//! the native module.
//!
//! It keeps the lifecycle the module defines in memory: a request is pending, a
//! person approves or denies it, an approval is reserved once and its outcome
//! recorded, a request past its window expires. It applies the user's posture the
//! way the module does (auto-approve everything, a tool grant, a flow grant) and
//! records what the gate asked, so a test can pin what was sent. It is not the
//! module: what the module itself answers is pinned by the module-backed tests
//! in `native/engine_tests.rs`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;

use super::engine::{
    ApprovalEngine, CallOrigin, CheckInput, Checked, Choice, Decided, Delivery, EngineError,
    Posture, PromptSurface, Sealed, Variant,
};
use super::types::{ApprovalAuditEntry, ApprovalDecision, ExecutionOutcome};
use crate::core::runtime::Tenant;

/// What the gate handed the engine for one call.
#[derive(Clone, Debug)]
pub(crate) struct Seen {
    pub(crate) tenant: Tenant,
    pub(crate) variant: Variant,
    pub(crate) posture: Posture,
    pub(crate) call_id: String,
    pub(crate) tool: String,
    pub(crate) args: serde_json::Value,
    pub(crate) thread: Option<String>,
    pub(crate) flow: Option<String>,
    pub(crate) origin: CallOrigin,
    pub(crate) ttl: Duration,
}

/// Where a stubbed request is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    Pending,
    Approved,
    Denied,
    Expired,
    Cancelled,
    Reserved,
    Executed(ExecutionOutcome),
}

#[derive(Clone, Debug)]
struct Row {
    tool: String,
    state: State,
    created_ms: u64,
    expires_ms: u64,
    decided_ms: Option<u64>,
    label: Option<String>,
    attribution: Option<&'static str>,
}

/// The scripted engine.
#[derive(Default)]
pub(crate) struct StubEngine {
    rows: Mutex<HashMap<String, Row>>,
    seen: Mutex<Vec<Seen>>,
    refuse: AtomicBool,
    fail_reserve: AtomicBool,
    always_pending: AtomicBool,
}

impl std::fmt::Debug for StubEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StubEngine").finish_non_exhaustive()
    }
}

impl Sealed for StubEngine {}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

impl StubEngine {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Everything the gate asked, in order.
    pub(crate) fn seen(&self) -> Vec<Seen> {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The state of a request, when the engine knows it.
    pub(crate) fn state(&self, request_id: &str) -> Option<State> {
        self.rows
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(request_id)
            .map(|row| row.state)
    }

    /// How a request was attributed when it was approved without a person.
    pub(crate) fn attribution(&self, request_id: &str) -> Option<&'static str> {
        self.rows
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(request_id)
            .and_then(|row| row.attribution)
    }

    /// Every request id, oldest first.
    pub(crate) fn request_ids(&self) -> Vec<String> {
        let rows = self.rows.lock().unwrap_or_else(PoisonError::into_inner);
        let mut ids: Vec<(&String, u64)> =
            rows.iter().map(|(id, row)| (id, row.created_ms)).collect();
        ids.sort_by_key(|(_, created)| *created);
        ids.into_iter().map(|(id, _)| id.clone()).collect()
    }

    /// Make every later `check` fail, as a module that cannot answer does.
    pub(crate) fn refuse_checks(&self, refuse: bool) {
        self.refuse.store(refuse, Ordering::SeqCst);
    }

    /// Make every later `check` ask for a person, whatever the posture says.
    pub(crate) fn always_pending(&self, pending: bool) {
        self.always_pending.store(pending, Ordering::SeqCst);
    }

    /// Make every later `reserve` fail.
    pub(crate) fn fail_reserve(&self, fail: bool) {
        self.fail_reserve.store(fail, Ordering::SeqCst);
    }

    /// Put a request in the store as an earlier launch left it: pending, with
    /// no parked call and no way to resume.
    pub(crate) fn leave_pending(&self, request_id: &str, tool: &str, expires_in: Duration) {
        let now = now_ms();
        self.rows
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(
                request_id.to_owned(),
                Row {
                    tool: tool.to_owned(),
                    state: State::Pending,
                    created_ms: now,
                    expires_ms: now + u64::try_from(expires_in.as_millis()).unwrap_or(u64::MAX),
                    decided_ms: None,
                    label: None,
                    attribution: None,
                },
            );
    }

    fn insert(&self, row: Row) -> String {
        let request_id = uuid::Uuid::new_v4().simple().to_string();
        self.rows
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(request_id.clone(), row);
        request_id
    }

    fn with<T>(&self, request_id: &str, change: impl FnOnce(&mut Row) -> T) -> Option<T> {
        self.rows
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get_mut(request_id)
            .map(change)
    }
}

/// The automatic approval the user's posture gives, if any.
fn automatic(input: &CheckInput) -> Option<&'static str> {
    let posture = &input.posture;
    if posture.auto_approve_all || !posture.require_approval {
        return Some("auto:host-policy");
    }
    if posture.tool_grants.iter().any(|t| t == &input.tool) {
        return Some("auto:tool-grant");
    }
    let flow = input.flow.as_deref()?;
    posture
        .flow_grants
        .iter()
        .any(|(f, t)| f == flow && t == &input.tool)
        .then_some("auto:flow-grant")
}

#[async_trait]
impl ApprovalEngine for StubEngine {
    async fn check(&self, input: CheckInput) -> Result<Checked, EngineError> {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Seen {
                tenant: input.tenant.clone(),
                variant: input.variant,
                posture: input.posture.clone(),
                call_id: input.call_id.clone(),
                tool: input.tool.clone(),
                args: input.args.clone(),
                thread: input.thread.clone(),
                flow: input.flow.clone(),
                origin: input.origin,
                ttl: input.ttl,
            });
        if self.refuse.load(Ordering::SeqCst) {
            return Err(EngineError::Unavailable("the stub engine refuses".into()));
        }
        let now = now_ms();
        let automatic = if self.always_pending.load(Ordering::SeqCst) {
            None
        } else {
            automatic(&input)
        };
        if let Some(attribution) = automatic {
            let request_id = self.insert(Row {
                tool: input.tool,
                state: State::Approved,
                created_ms: now,
                expires_ms: now + 600_000,
                decided_ms: Some(now),
                label: None,
                attribution: Some(attribution),
            });
            return Ok(Checked::Approved { request_id });
        }
        // The module's window: ten minutes, three for a copilot or sub-agent,
        // and a caller may only shorten it.
        let window = match input.origin {
            CallOrigin::Copilot | CallOrigin::SubAgent => Duration::from_secs(180),
            CallOrigin::Operator => Duration::from_secs(600),
        }
        .min(input.ttl);
        let expires_ms = now + u64::try_from(window.as_millis()).unwrap_or(u64::MAX);
        let request_id = self.insert(Row {
            tool: input.tool.clone(),
            state: State::Pending,
            created_ms: now,
            expires_ms,
            decided_ms: None,
            label: None,
            attribution: None,
        });
        let delivered = input
            .prompt
            .deliver(Delivery {
                call_id: input.call_id,
                request_id: request_id.clone(),
                tool_name: input.tool,
                summary: input.summary,
                args_redacted: input.args,
                created_at_ms: now,
                expires_at_ms: expires_ms,
            })
            .await;
        if let Err(error) = delivered {
            self.with(&request_id, |row| row.state = State::Cancelled);
            return Err(EngineError::Unavailable(error));
        }
        Ok(Checked::Pending {
            request_id,
            expires_at_ms: expires_ms,
        })
    }

    async fn decide(
        &self,
        request_id: &str,
        choice: Choice,
        label: &str,
    ) -> Result<Decided, EngineError> {
        let now = now_ms();
        self.with(request_id, |row| {
            if row.state != State::Pending || now >= row.expires_ms {
                return Err(EngineError::NotPending);
            }
            row.decided_ms = Some(now);
            row.label = Some(label.to_owned());
            match choice {
                Choice::Once => {
                    row.state = State::Approved;
                    Ok(Decided::Approved)
                }
                Choice::Deny => {
                    row.state = State::Denied;
                    Ok(Decided::Denied)
                }
            }
        })
        .unwrap_or(Err(EngineError::NotPending))
    }

    async fn reserve(&self, request_id: &str) -> Result<(), EngineError> {
        if self.fail_reserve.load(Ordering::SeqCst) {
            return Err(EngineError::Unavailable(
                "the stub engine cannot reserve".into(),
            ));
        }
        self.with(request_id, |row| {
            if row.state == State::Approved {
                row.state = State::Reserved;
                Ok(())
            } else {
                Err(EngineError::Unavailable(
                    "the request is not approved".into(),
                ))
            }
        })
        .unwrap_or(Err(EngineError::NotPending))
    }

    async fn record_execution(
        &self,
        request_id: &str,
        outcome: ExecutionOutcome,
    ) -> Result<(), EngineError> {
        self.with(request_id, |row| match row.state {
            State::Reserved => {
                row.state = State::Executed(outcome);
                Ok(())
            }
            State::Executed(done) if done == outcome => Ok(()),
            _ => Err(EngineError::Unavailable(
                "the request was not reserved".into(),
            )),
        })
        .unwrap_or(Err(EngineError::NotPending))
    }

    async fn expire(&self, request_id: &str) -> Result<bool, EngineError> {
        let now = now_ms();
        self.with(request_id, |row| {
            if row.state == State::Pending {
                if now < row.expires_ms {
                    return Err(EngineError::Unavailable("the request is still live".into()));
                }
                row.state = State::Expired;
                row.decided_ms = Some(now);
                return Ok(true);
            }
            Ok(false)
        })
        .unwrap_or(Err(EngineError::NotPending))
    }

    fn release(&self, _request_id: &str) {}

    async fn cancel(&self, request_id: &str) -> Result<(), EngineError> {
        self.with(request_id, |row| {
            if matches!(row.state, State::Pending | State::Approved) {
                row.state = State::Cancelled;
                row.decided_ms = row.decided_ms.or_else(|| Some(now_ms()));
                Ok(())
            } else {
                Err(EngineError::NotPending)
            }
        })
        .unwrap_or(Err(EngineError::NotPending))
    }

    async fn is_decided(&self, request_id: &str) -> Result<Option<bool>, EngineError> {
        Ok(self.state(request_id).map(|state| state != State::Pending))
    }

    async fn history(&self, limit: usize) -> Result<Vec<ApprovalAuditEntry>, EngineError> {
        let rows = self.rows.lock().unwrap_or_else(PoisonError::into_inner);
        let mut decided: Vec<(&String, &Row)> = rows
            .iter()
            .filter(|(_, row)| row.state != State::Pending)
            .collect();
        decided.sort_by_key(|(_, row)| std::cmp::Reverse(row.decided_ms));
        Ok(decided
            .into_iter()
            .take(limit)
            .map(|(id, row)| {
                let at = |ms: u64| {
                    chrono::DateTime::from_timestamp_millis(i64::try_from(ms).unwrap_or(0))
                        .unwrap_or_default()
                };
                let approved = matches!(
                    row.state,
                    State::Approved | State::Reserved | State::Executed(_)
                );
                ApprovalAuditEntry {
                    request_id: id.clone(),
                    tool_name: row.tool.clone(),
                    action_summary: String::new(),
                    args_redacted: serde_json::Value::Null,
                    created_at: at(row.created_ms),
                    expires_at: Some(at(row.expires_ms)),
                    decided_at: at(row.decided_ms.unwrap_or(row.created_ms)),
                    decision: if approved {
                        row.label
                            .as_deref()
                            .and_then(ApprovalDecision::from_str)
                            .unwrap_or(ApprovalDecision::ApproveOnce)
                    } else {
                        ApprovalDecision::Deny
                    },
                    decided_by: row.attribution.map(str::to_owned),
                    judge: None,
                }
            })
            .collect())
    }
}

/// A prompt surface that records what it was shown, for the tests of an engine.
#[derive(Default)]
pub(crate) struct Recording {
    pub(crate) shown: Mutex<Vec<Delivery>>,
}

#[async_trait]
impl PromptSurface for Recording {
    async fn deliver(&self, delivery: Delivery) -> Result<(), String> {
        self.shown
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(delivery);
        Ok(())
    }
}
