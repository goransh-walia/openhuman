use std::path::PathBuf;

use super::types::{DurableSubagentSession, SubagentSessionStore};

impl SubagentSessionStore {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }

    /// The pre-documents `<workspace>/.openhuman/subagent_sessions.json`.
    fn legacy_path(&self) -> PathBuf {
        self.workspace_dir
            .join(".openhuman")
            .join("subagent_sessions.json")
    }

    /// Imports the pre-documents session file once: its sessions are written
    /// (an already stored newer copy wins) and the file is renamed to
    /// `subagent_sessions.json.migrated`, so it is never imported again and
    /// stays recoverable. A missing file is the common case and a no-op.
    fn import_legacy(&self, docs: &super::store_documents::Docs) -> Result<(), String> {
        let path = self.legacy_path();
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(format!("failed to read legacy subagent sessions: {err}")),
        };
        let sessions: Vec<DurableSubagentSession> = serde_json::from_str(&raw)
            .map_err(|err| format!("failed to parse legacy subagent sessions: {err}"))?;
        docs.save(&sessions).map_err(|err| format!("{err:#}"))?;
        log::debug!(
            "[subagent_sessions] imported legacy sessions count={}",
            sessions.len()
        );
        if let Err(err) = std::fs::rename(&path, path.with_extension("json.migrated")) {
            log::warn!("[subagent_sessions] could not retire legacy file: {err}");
        }
        Ok(())
    }

    pub fn load(&self) -> Result<Vec<DurableSubagentSession>, String> {
        let docs = super::store_documents::current(&self.workspace_dir)
            .map_err(|err| format!("{err:#}"))?;
        self.import_legacy(&docs)?;
        docs.load().map_err(|err| format!("{err:#}"))
    }

    pub fn save(&self, sessions: &[DurableSubagentSession]) -> Result<(), String> {
        let docs = super::store_documents::current(&self.workspace_dir)
            .map_err(|err| format!("{err:#}"))?;
        self.import_legacy(&docs)?;
        docs.save(sessions).map_err(|err| format!("{err:#}"))
    }
}
