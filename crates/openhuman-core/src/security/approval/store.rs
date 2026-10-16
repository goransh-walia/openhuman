//! What the host keeps beside the module's approval rows.
//!
//! The lifecycle of an approval request is the TinySecurity module's; its rows
//! live in `native/rows.rs` and are read through the engine. This module keeps
//! the host's own documents on the `tinystoragedrivers` document port
//! ([`store_documents`](super::store_documents)), under the current call's
//! storage scope (the acting agent; `local` on a single-user host; refused in
//! SaaS mode without one):
//!
//! - per-flow trust: the "approve always for this flow" grants, which reach the
//!   engine as the flow grants of a workflow's next call; and
//! - recorded decisions: a flow pre-authorization's born-decided audit row and
//!   the history older installs wrote (the `approval/approval.db` they used is
//!   read once, read-only, by the storage importer, `store_import.rs`).

// Flow pre-authorization + per-flow tool trust persistence.
#[path = "store_flow_trust.rs"]
mod store_flow_trust;
pub use store_flow_trust::{
    delete_flow_trust, insert_flow_trust, is_flow_tool_trusted, list_all_flow_trust,
    list_flow_trust, list_recorded_decisions, record_flow_preauthorization,
};

pub use super::store_import::PRE_MIGRATION_SESSION_ID;

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
