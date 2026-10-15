//! Fold the legacy security tables into the single `[security]` table.
//!
//! Runs on the parsed document before the typed decode, on every load, so the
//! result does not depend on whether the workspace crossed schema version 15:
//!
//! * `[security.autonomy]`, `[security.sandbox]` and `[security.privacy]` are
//!   the canonical spelling. They are overlaid key by key onto the legacy
//!   top-level `[autonomy]`, `[sandbox]` and `[privacy]` tables, and the
//!   `[security.*]` value wins. Consumers keep reading `Config::{autonomy,
//!   sandbox, privacy}`.
//! * A document with neither spelling is untouched, so the policy stays off
//!   (`autonomy.enabled` defaults to `false`).
//! * `autonomy.max_cost_per_day_cents` is deleted: nothing enforced it, and the
//!   policy module no longer advertises a cost budget (tinysecurity ADR 0006).
//!   Each removal is reported through [`Notice`] and logged.
//! * `sandbox.backend = "firejail" | "bubblewrap"` becomes `"auto"` and
//!   `sandbox.firejail_args` is deleted. Neither backend ever had an execution
//!   path (selecting one made every command fail), so the choice is migrated to
//!   the TinyBox-selected jail, which is the isolation they asked for and keeps
//!   the fail-closed floor for untrusted code. Both the legacy `[sandbox]` and
//!   the `[security.sandbox]` spelling are rewritten, each with a [`Notice`].

use toml::Value;

/// The three tables that move under `[security]`.
const FOLDED: [&str; 3] = ["autonomy", "sandbox", "privacy"];

/// A user-visible record of something the rewrite changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Notice {
    /// A retired `max_cost_per_day_cents` key was dropped from this table.
    DroppedMaxCost { table: &'static str },
    /// `[security.<table>]` was overlaid onto the legacy `[<table>]`.
    Overlaid { table: &'static str },
    /// A retired sandbox backend (`firejail`, `bubblewrap`) was moved to `auto`.
    RetiredSandboxBackend {
        table: &'static str,
        from: &'static str,
    },
    /// The retired `firejail_args` key was dropped from this table.
    DroppedFirejailArgs { table: &'static str },
}

/// Retire the sandbox backends and arguments that never had an executor.
fn retire_sandbox_keys(
    table: &mut toml::map::Map<String, Value>,
    name: &'static str,
) -> Vec<Notice> {
    let mut notices = Vec::new();
    if table.remove("firejail_args").is_some() {
        notices.push(Notice::DroppedFirejailArgs { table: name });
    }
    let retired = match table.get("backend").and_then(Value::as_str) {
        Some("firejail") => Some("firejail"),
        Some("bubblewrap") => Some("bubblewrap"),
        _ => None,
    };
    if let Some(from) = retired {
        table.insert("backend".to_owned(), Value::String("auto".to_owned()));
        notices.push(Notice::RetiredSandboxBackend { table: name, from });
    }
    notices
}

pub(crate) fn rewrite_document(document: &mut Value) -> Vec<Notice> {
    let mut notices = Vec::new();
    let Some(root) = document.as_table_mut() else {
        return notices;
    };
    if let Some(Value::Table(autonomy)) = root.get_mut("autonomy") {
        if autonomy.remove("max_cost_per_day_cents").is_some() {
            notices.push(Notice::DroppedMaxCost { table: "autonomy" });
        }
    }
    if let Some(Value::Table(sandbox)) = root.get_mut("sandbox") {
        notices.extend(retire_sandbox_keys(sandbox, "sandbox"));
    }
    let mut overlays = Vec::new();
    if let Some(Value::Table(security)) = root.get_mut("security") {
        for name in FOLDED {
            if let Some(Value::Table(mut table)) = security.remove(name) {
                if name == "autonomy" && table.remove("max_cost_per_day_cents").is_some() {
                    notices.push(Notice::DroppedMaxCost {
                        table: "security.autonomy",
                    });
                }
                if name == "sandbox" {
                    notices.extend(retire_sandbox_keys(&mut table, "security.sandbox"));
                }
                overlays.push((name, table));
            }
        }
    }
    for (name, table) in overlays {
        let target = root
            .entry(name.to_string())
            .or_insert_with(|| Value::Table(Default::default()));
        if !target.is_table() {
            *target = Value::Table(Default::default());
        }
        if let Value::Table(target) = target {
            for (key, value) in table {
                target.insert(key, value);
            }
        }
        notices.push(Notice::Overlaid { table: name });
    }
    for notice in &notices {
        match notice {
            Notice::DroppedMaxCost { table } => log::warn!(
                "[migrations][security] removed retired {table}.max_cost_per_day_cents: \
                 no cost budget is enforced"
            ),
            Notice::RetiredSandboxBackend { table, from } => log::warn!(
                "[migrations][security] {table}.backend = \"{from}\" has no execution backend; \
                 moved to \"auto\" (the TinyBox-selected jail)"
            ),
            Notice::DroppedFirejailArgs { table } => log::warn!(
                "[migrations][security] removed retired {table}.firejail_args: no Firejail backend exists"
            ),
            Notice::Overlaid { table } => {
                log::debug!("[migrations][security] [security.{table}] overlaid onto [{table}]")
            }
        }
    }
    notices
}

#[cfg(test)]
#[path = "security_policy_tests.rs"]
mod tests;
