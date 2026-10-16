//! The host's own approval documents on the `tinystoragedrivers` document port.
//!
//! The lifecycle of an approval request (pending, decided, expired, executed) is
//! the TinySecurity module's, and its rows live in `native/rows.rs`. This file
//! keeps what the host owns beside them:
//!
//! | Collection | Document id | Holds |
//! | --- | --- | --- |
//! | `approvals` | request id | the history the module's rows replaced: what older installs wrote, plus a flow pre-authorization's born-decided audit row. Read for history, never decided again |
//! | `approval_flow_trust` | `(flow_id, tool_name)` | one "approve always for this flow" grant, which reaches the engine as a flow grant |
//!
//! `args_redacted` and `source_context` are stored as JSON strings, so a
//! MongoDB-backed port never meets an arbitrary key.

use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::config::Config;
use crate::storage::local;
use crate::storage::{block_on, DocumentStore, DocumentStoreExt, ScopedStorage, StorageError};
use tinystoragedrivers::{
    CollectionSpec, ErrorKind, Filter, IndexSpec, Precondition, Query, Sort, Versioned,
};

use super::types::{
    ApprovalAuditEntry, ApprovalDecision, ApprovalSourceContext, ExecutionOutcome, PendingApproval,
};

pub(super) const APPROVALS: &str = "approvals";
pub(super) const FLOW_TRUST: &str = "approval_flow_trust";

const DOMAIN: &str = "approval::store";

/// The approval store over one scoped document handle.
#[derive(Clone, Debug)]
pub(super) struct Docs {
    docs: Arc<dyn DocumentStore>,
}

/// The document store for this call: the storage backend, in the acting scope.
///
/// # Errors
///
/// When the storage scope cannot be resolved — in SaaS mode with no acting
/// agent — so the call fails rather than reading a shared bucket.
pub(super) fn current(config: &Config) -> Result<Docs> {
    let opened = local::open(DOMAIN, &config.workspace_dir)?;
    Ok(Docs::new(&opened.scoped()?))
}

/// The store of embedded agent `agent`'s scope, whichever scope the caller
/// runs in.
pub(super) fn for_agent(config: &Config, agent: &str) -> Result<Docs> {
    let opened = local::open(DOMAIN, &config.workspace_dir)?;
    let scoped = opened
        .backend
        .for_scope(&crate::storage::scope_for_agent(agent))
        .map_err(|error| anyhow!("[approval::store] open the agent's scope: {error}"))?;
    Ok(Docs::new(&scoped))
}

/// The legacy `approval.db`, as the storage importer reads it.
pub(crate) fn import_source() -> crate::storage::import::Source {
    crate::storage::import::Source {
        domain: "approval",
        db: "approval/approval.db",
        collections,
        read_legacy: |scratch, _real| super::store_import::read(scratch),
        rekey: |record, _config| record,
    }
}

/// The collections the approval store keeps.
pub(super) fn collections() -> Vec<CollectionSpec> {
    vec![
        CollectionSpec::new(APPROVALS)
            .index(IndexSpec::new("by_pending", ["pending", "created_at"]))
            .index(IndexSpec::new("by_decided", ["pending", "decided_at"]))
            .index(IndexSpec::new("by_session", ["session_id", "pending"])),
        CollectionSpec::new(FLOW_TRUST).index(IndexSpec::new("by_flow", ["flow_id", "tool_name"])),
    ]
}

fn storage(error: StorageError) -> anyhow::Error {
    anyhow!("[approval::store] storage: {error}")
}

/// Fixed-width (nanosecond, `Z`) so the strings sort in time order.
pub(super) fn rfc3339(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}

/// The comparable expiry key: nanoseconds since the epoch, falling back to
/// milliseconds for instants beyond the i64-nanosecond range (year 2262).
pub(super) fn expiry_key(at: DateTime<Utc>) -> i64 {
    at.timestamp_nanos_opt()
        .unwrap_or_else(|| at.timestamp_millis().saturating_mul(1_000_000))
}

fn parse_time(raw: Option<&str>) -> Option<DateTime<Utc>> {
    raw.and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|at| at.with_timezone(&Utc))
}

fn text<'a>(doc: &'a Value, field: &str) -> Option<&'a str> {
    doc.get(field).and_then(Value::as_str)
}

/// A stored request as a [`PendingApproval`].
fn to_pending(stored: &Versioned<Value>) -> PendingApproval {
    let doc = &stored.doc;
    PendingApproval {
        request_id: stored.id.clone(),
        tool_name: text(doc, "tool_name").unwrap_or_default().to_string(),
        action_summary: text(doc, "action_summary").unwrap_or_default().to_string(),
        args_redacted: text(doc, "args_redacted")
            .and_then(|raw| serde_json::from_str(raw).ok())
            .unwrap_or(Value::Null),
        created_at: parse_time(text(doc, "created_at")).unwrap_or_else(Utc::now),
        expires_at: parse_time(text(doc, "expires_at")),
        source_context: text(doc, "source_context").and_then(|raw| {
            serde_json::from_str::<ApprovalSourceContext>(raw)
                .map_err(|error| {
                    tracing::warn!(
                        %error,
                        "[approval::store] failed to decode source_context JSON — treating as absent"
                    );
                })
                .ok()
        }),
        tool_call_id: text(doc, "tool_call_id").map(str::to_string),
        agent_id: text(doc, "agent_id").map(str::to_string),
    }
}

/// A decided request as an [`ApprovalAuditEntry`]; `None` if it is not
/// decided or its decision is unknown.
fn to_audit(stored: &Versioned<Value>) -> Option<ApprovalAuditEntry> {
    let pending = to_pending(stored);
    let decided_at = parse_time(text(&stored.doc, "decided_at"))?;
    let decision = ApprovalDecision::from_str(text(&stored.doc, "decision")?)?;
    Some(ApprovalAuditEntry {
        request_id: pending.request_id,
        tool_name: pending.tool_name,
        action_summary: pending.action_summary,
        args_redacted: pending.args_redacted,
        created_at: pending.created_at,
        expires_at: pending.expires_at,
        decided_at,
        decision,
        decided_by: text(&stored.doc, "decided_by").map(str::to_owned),
        judge: stored
            .doc
            .get("judge")
            .and_then(|value| serde_json::from_value(value.clone()).ok()),
    })
}

impl Docs {
    pub(super) fn new(scoped: &ScopedStorage) -> Self {
        Self {
            docs: Arc::clone(scoped.documents()),
        }
    }

    /// Runs `op` against the store from synchronous code.
    fn run<T, F, Fut>(&self, op: F) -> Result<T>
    where
        F: FnOnce(Arc<dyn DocumentStore>) -> Fut,
        Fut: std::future::Future<Output = Result<T, StorageError>> + Send + 'static,
        T: Send + 'static,
    {
        let docs = Arc::clone(&self.docs);
        let future = op(Arc::clone(&docs));
        block_on(async move {
            declare(&docs).await?;
            future.await
        })
        .map_err(storage)
    }

    /// Inserts a request that is decided from birth (a flow
    /// pre-authorization): it never appears pending, only in the audit
    /// trail.
    pub(super) fn insert_decided(
        &self,
        tool_name: &str,
        action_summary: &str,
        session_id: &str,
        source_context: &ApprovalSourceContext,
        decision: ApprovalDecision,
    ) -> Result<()> {
        let now = rfc3339(Utc::now());
        let doc = json!({
            "tool_name": tool_name,
            "action_summary": action_summary,
            "args_redacted": "{}",
            "session_id": session_id,
            "created_at": now,
            "source_context": serde_json::to_string(source_context)
                .context("[approval::store] serialize preauthorization source_context")?,
            "pending": false,
            "decided_at": now,
            "decision": decision.as_str(),
        });
        let id = uuid::Uuid::new_v4().to_string();
        self.run(|docs| async move {
            docs.put(APPROVALS, &id, doc, Precondition::Absent)
                .await
                .map(|_| ())
        })
    }

    pub(super) fn list_recent_decisions(&self, limit: usize) -> Result<Vec<ApprovalAuditEntry>> {
        self.run(|docs| async move {
            let query =
                Query::filter(Filter::eq("pending", false).and(Filter::exists("decision", true)))
                    .sort(Sort::desc("decided_at"))
                    .limit(limit);
            let page = docs.query(APPROVALS, &query).await?;
            Ok(page.items.iter().filter_map(to_audit).collect())
        })
    }

    pub(super) fn insert_flow_trust(&self, flow_id: &str, tool_name: &str) -> Result<()> {
        let id = trust_id(flow_id, tool_name);
        let doc = json!({
            "flow_id": flow_id,
            "tool_name": tool_name,
            "created_at": rfc3339(Utc::now()),
        });
        self.run(|docs| async move {
            // `INSERT OR IGNORE`: re-granting is a no-op.
            match docs.put(FLOW_TRUST, &id, doc, Precondition::Absent).await {
                Ok(_) => Ok(()),
                Err(error) if error.kind() == ErrorKind::Conflict => Ok(()),
                Err(error) => Err(error),
            }
        })
    }

    pub(super) fn list_flow_trust(&self, flow_id: &str) -> Result<Vec<String>> {
        let flow = flow_id.to_string();
        self.run(|docs| async move {
            let query = Query::filter(Filter::eq("flow_id", flow)).sort(Sort::asc("tool_name"));
            Ok(docs
                .query_all(FLOW_TRUST, &query)
                .await?
                .iter()
                .filter_map(|grant| text(&grant.doc, "tool_name").map(str::to_string))
                .collect())
        })
    }

    pub(super) fn delete_flow_trust(
        &self,
        flow_id: &str,
        tool_names: Option<&[String]>,
    ) -> Result<usize> {
        let flow = flow_id.to_string();
        let names = tool_names.map(<[String]>::to_vec);
        self.run(|docs| async move {
            let removed = match names {
                None => {
                    docs.delete_where(FLOW_TRUST, &Filter::eq("flow_id", flow))
                        .await?
                }
                Some(names) => {
                    let mut removed = 0;
                    for name in names {
                        if docs
                            .delete(FLOW_TRUST, &trust_id(&flow, &name), Precondition::None)
                            .await?
                        {
                            removed += 1;
                        }
                    }
                    removed
                }
            };
            Ok(usize::try_from(removed).unwrap_or(usize::MAX))
        })
    }

    pub(super) fn is_flow_tool_trusted(&self, flow_id: &str, tool_name: &str) -> Result<bool> {
        let id = trust_id(flow_id, tool_name);
        self.run(|docs| async move { Ok(docs.get(FLOW_TRUST, &id).await?.is_some()) })
    }
}

/// The id of a `(flow_id, tool_name)` grant: length-prefixed, so no two
/// pairs collide.
pub(super) fn trust_id(flow_id: &str, tool_name: &str) -> String {
    format!("{}:{flow_id}/{tool_name}", flow_id.len())
}

async fn declare(docs: &Arc<dyn DocumentStore>) -> Result<(), StorageError> {
    for spec in collections() {
        docs.ensure_collection(&spec).await?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "store_documents_tests.rs"]
mod tests;
