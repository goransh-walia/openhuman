//! Fail-closed gate for tinyflows' out-of-process script runner (issue #7328,
//! phase 5).
//!
//! The host's `code` node runs through the sandbox (`code.rs`), and no host
//! capability wires tinyflows' `ProcessCodeRunner` or `ProcessShellRunner`. Both
//! of those would spawn an interpreter as an unjailed child of this process
//! unless a [`tinyflows::caps::host::ScriptSpawner`] is installed. This process
//! installs none, so it marks the spawner as required: any script run that
//! reaches the runner is refused instead of falling back to a direct spawn.

/// Refuse every tinyflows script run that has no jailing spawner installed.
/// Idempotent; called whenever flow capabilities are built.
pub(crate) fn require_jailed_scripts() {
    tracing::debug!(target: "flows", "[flows] script runner: spawner required, direct spawn refused");
    tinyflows::caps::host::require_script_spawner(true);
}

#[cfg(test)]
#[path = "script_gate_tests.rs"]
mod tests;
