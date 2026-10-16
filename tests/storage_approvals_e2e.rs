//! What the host keeps beside the approval engine's rows, on a configured storage
//! backend, end to end through its public functions: per-flow trust and the
//! recorded decision history.
//!
//! Its own test binary because it installs a backend into the process-wide
//! storage slot, which would reroute every other suite's approvals in a
//! shared process.

use openhuman_core::config::Config;
use openhuman_core::security::approval::store;
use openhuman_core::security::approval::types::ApprovalDecision;

#[macro_use]
#[path = "support/storage_drivers.rs"]
mod storage_drivers;

use storage_drivers::Case;

fn a_configured_backend_holds_the_approvals_instead_of_approval_db(case: Case) {
    let workspace = tempfile::tempdir().unwrap();
    let config = Config {
        workspace_dir: workspace.path().to_path_buf(),
        ..Config::default()
    };
    case.install();

    store::insert_flow_trust(&config, "flow-1", "shell").unwrap();
    store::record_flow_preauthorization(&config, "flow-1", "shell", "session").unwrap();
    assert!(store::is_flow_tool_trusted(&config, "flow-1", "shell").unwrap());
    assert_eq!(
        store::list_all_flow_trust(&config).unwrap(),
        [("flow-1".to_string(), "shell".to_string())]
    );
    let recorded = store::list_recorded_decisions(&config, 10).unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].tool_name, "shell");
    assert_eq!(recorded[0].decision, ApprovalDecision::ApproveAlwaysForFlow);

    assert!(
        !workspace
            .path()
            .join("approval")
            .join("approval.db")
            .exists(),
        "nothing was written to the legacy database"
    );

    // Without a backend the workspace's own default database is used, and the
    // configured backend's records are not in it.
    assert!(openhuman_core::storage::clear());
    assert!(!store::is_flow_tool_trusted(&config, "flow-1", "shell").unwrap());
    assert!(store::list_recorded_decisions(&config, 10).unwrap().is_empty());
    assert!(workspace.path().join("storage").join("storage.db").exists());
}

driver_cases!(sync a_configured_backend_holds_the_approvals_instead_of_approval_db);
