use super::*;
use crate::config::Config;
use crate::security::approval::types::ApprovalDecision;
use tempfile::TempDir;

fn test_config() -> (Config, TempDir) {
    let dir = TempDir::new().unwrap();
    let config = Config {
        workspace_dir: dir.path().to_path_buf(),
        ..Config::default()
    };
    (config, dir)
}

#[path = "store_flow_trust_tests.rs"]
mod flow_trust_tests;
