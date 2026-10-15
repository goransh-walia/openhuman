use super::*;
use tinybox_bus::CommandClass;

// -- AutonomyLevel ------------------------------------------------

#[test]
fn autonomy_default_is_supervised() {
    assert_eq!(AutonomyLevel::default(), AutonomyLevel::Supervised);
}

#[test]
fn autonomy_serde_roundtrip() {
    let json = serde_json::to_string(&AutonomyLevel::Full).unwrap();
    assert_eq!(json, "\"full\"");
    let parsed: AutonomyLevel = serde_json::from_str("\"readonly\"").unwrap();
    assert_eq!(parsed, AutonomyLevel::ReadOnly);
    let parsed2: AutonomyLevel = serde_json::from_str("\"supervised\"").unwrap();
    assert_eq!(parsed2, AutonomyLevel::Supervised);
}

#[test]
fn can_act_readonly_false() {
    assert!(!readonly_policy().can_act());
}

#[test]
fn can_act_supervised_true() {
    assert!(default_policy().can_act());
}

#[test]
fn can_act_full_true() {
    assert!(full_policy().can_act());
}

#[test]
fn enforce_tool_operation_read_allowed_in_readonly_mode() {
    let p = readonly_policy();
    assert!(p
        .enforce_tool_operation(ToolOperation::Read, "memory_recall")
        .is_ok());
}

#[test]
fn enforce_tool_operation_act_blocked_in_readonly_mode() {
    let p = readonly_policy();
    let err = p
        .enforce_tool_operation(ToolOperation::Act, "memory_store")
        .unwrap_err();
    assert!(err.contains("read-only mode"));
}

#[test]
fn enforce_tool_operation_act_uses_rate_budget() {
    let p = SecurityPolicy {
        max_actions_per_hour: 0,
        ..default_policy()
    };
    let err = p
        .enforce_tool_operation(ToolOperation::Act, "memory_store")
        .unwrap_err();
    assert!(err.contains("Rate limit exceeded"));
}

#[test]
fn action_budget_error_mentions_limit_and_settings() {
    let p = SecurityPolicy {
        max_actions_per_hour: 0,
        ..default_policy()
    };

    let err = p
        .enforce_tool_operation(ToolOperation::Act, "write_file")
        .unwrap_err();

    assert!(err.contains("Rate limit exceeded: action budget exhausted"));
    assert!(err.contains("0 actions/hour"));
    assert!(err.contains("Settings -> Advanced -> Agent autonomy"));
}

// -- is_command_allowed -------------------------------------------

// -- cross-platform always-forbidden hardening (Phase E) ----------

#[test]
fn always_forbidden_blocks_credential_stores_case_insensitively() {
    use std::path::Path;
    for p in [
        "/home/u/.ssh/id_rsa",
        "/home/u/.SSH/id_rsa", // case-insensitive
        "C:\\Users\\u\\.ssh\\id_rsa",
        "/home/u/.gnupg/x",
        "/home/u/.aws/credentials",
        "/home/u/.azure/x",
        "/home/u/.kube/config",
        "/Users/u/Library/Keychains/login.keychain",
        "C:\\Users\\u\\AppData\\Roaming\\Microsoft\\Protect\\x",
        "C:\\Users\\u\\AppData\\Local\\Microsoft\\Credentials\\x",
    ] {
        assert!(SecurityPolicy::is_always_forbidden(Path::new(p)), "{p}");
    }
}
