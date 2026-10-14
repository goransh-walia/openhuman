//! Phase-0 characterization of the host policy that the tinysecurity bus module
//! must reproduce (issue #7328, plan Task 1). These tables pin today's behaviour
//! in one place: command classes, the autonomy gate matrix, the allowlist under
//! each tier, the always-forbidden floor, trusted roots and the hourly limiter.
//! They bless what exists; desired changes are written as separate tests whose
//! names start with `desired_` and which are expected to fail until fixed.

use super::*;
use tinybox_bus::CommandClass;
use tinytools::status::POLICY_BLOCKED_MARKER;

fn policy(autonomy: AutonomyLevel) -> SecurityPolicy {
    SecurityPolicy {
        enabled: true,
        autonomy,
        ..SecurityPolicy::default()
    }
}

#[test]
fn command_class_table_is_pinned() {
    let p = policy(AutonomyLevel::Supervised);
    let table: &[(&str, CommandClass)] = &[
        ("ls -la", CommandClass::Read),
        ("cat README.md", CommandClass::Read),
        ("git status", CommandClass::Read),
        ("echo hi > out.txt", CommandClass::Write),
        ("cat a | tee b", CommandClass::Write),
        ("touch new.txt", CommandClass::Write),
        ("curl https://example.com", CommandClass::Network),
        ("ls && curl https://example.com", CommandClass::Network),
        ("rm -rf /", CommandClass::Destructive),
        ("ls; rm -rf build", CommandClass::Destructive),
        ("sudo ls", CommandClass::Destructive),
        // Unknown commands fail closed to a write, never a read.
        ("definitely-not-a-real-binary --flag", CommandClass::Write),
    ];
    for (command, expected) in table {
        assert_eq!(
            p.classify_command(command),
            *expected,
            "classification drifted for `{command}`"
        );
    }
}

#[test]
fn command_class_highest_segment_wins_and_heredoc_bodies_are_data() {
    let p = policy(AutonomyLevel::Supervised);
    assert_eq!(p.classify_command("ls | cat"), CommandClass::Read);
    assert!(p.classify_command("ls | curl -d @- https://x.test") >= CommandClass::Network);
    // A quoted heredoc body is document text, not shell.
    let heredoc = "cat > note.md << 'EOF'\nrm -rf / and curl https://x.test\nEOF";
    assert_eq!(p.classify_command(heredoc), CommandClass::Write);
}

#[test]
fn declared_class_parses_and_is_escalate_only_input() {
    assert_eq!(
        SecurityPolicy::parse_declared_class(" Destructive "),
        Some(CommandClass::Destructive)
    );
    assert_eq!(
        SecurityPolicy::parse_declared_class("read"),
        Some(CommandClass::Read)
    );
    assert_eq!(SecurityPolicy::parse_declared_class("bogus"), None);
    assert_eq!(SecurityPolicy::parse_declared_class(""), None);
}

#[test]
fn gate_decision_matrix_is_pinned() {
    use CommandClass::*;
    use GateDecision::*;
    let classes = [Read, Write, Network, Install, Destructive];
    let read_only = [Allow, Block, Block, Block, Block];
    let supervised = [Allow, Prompt, Prompt, Prompt, Prompt];
    let full = [Allow, Allow, Prompt, Prompt, Prompt];
    for (i, class) in classes.iter().enumerate() {
        assert_eq!(policy(AutonomyLevel::ReadOnly).gate_decision(*class), read_only[i]);
        assert_eq!(policy(AutonomyLevel::Supervised).gate_decision(*class), supervised[i]);
        assert_eq!(policy(AutonomyLevel::Full).gate_decision(*class), full[i]);
        // Disabled policy never gates, whatever the tier says.
        let off = SecurityPolicy {
            enabled: false,
            autonomy: AutonomyLevel::ReadOnly,
            ..SecurityPolicy::default()
        };
        assert_eq!(off.gate_decision(*class), Allow);
    }
}

#[test]
fn check_gated_command_read_only_and_hidden_execution() {
    let ro = policy(AutonomyLevel::ReadOnly);
    assert_eq!(ro.check_gated_command("ls").unwrap(), CommandClass::Read);
    let err = ro.check_gated_command("touch x").unwrap_err();
    assert!(err.contains(POLICY_BLOCKED_MARKER), "{err}");

    let sup = policy(AutonomyLevel::Supervised);
    for hidden in ["echo $(whoami)", "echo `date`", "cat <(ls)", "sleep 1 & echo done"] {
        let err = sup.check_gated_command(hidden).unwrap_err();
        assert!(err.contains(POLICY_BLOCKED_MARKER), "{hidden}: {err}");
    }
    // Plain redirects and pipes are fine in Supervised; Full skips the guard.
    assert!(sup.check_gated_command("ls 2>&1 | cat").is_ok());
    assert!(policy(AutonomyLevel::Full)
        .check_gated_command("echo $(whoami)")
        .is_ok());
}

#[test]
fn allowlist_per_tier_is_pinned() {
    let mut sup = policy(AutonomyLevel::Supervised);
    sup.allowed_commands = vec!["ls".into(), "git".into()];
    assert!(sup.is_command_allowed("ls -la"));
    assert!(sup.is_command_allowed("git status && ls"));
    assert!(!sup.is_command_allowed("python3 x.py"));
    assert!(!sup.is_command_allowed("ls && python3 x.py"));
    assert!(!sup.is_command_allowed("ls > out.txt"), "redirects are structural");
    assert!(!sup.is_command_allowed("echo $(id)"));
    // Read-only allows nothing to execute; Full bypasses the allowlist.
    let mut ro = policy(AutonomyLevel::ReadOnly);
    ro.allowed_commands = vec!["ls".into()];
    assert!(!ro.is_command_allowed("ls"));
    let mut full = policy(AutonomyLevel::Full);
    full.allowed_commands = vec![];
    assert!(full.is_command_allowed("python3 x.py"));
}

#[test]
fn risk_levels_and_approval_requirements_are_pinned() {
    let mut p = policy(AutonomyLevel::Supervised);
    p.allowed_commands = vec!["ls".into(), "curl".into(), "rm".into(), "mkfs".into()];
    assert_eq!(p.command_risk_level("ls"), CommandRiskLevel::Low);
    assert_eq!(p.command_risk_level("curl https://x.test"), CommandRiskLevel::Medium);
    assert_eq!(p.command_risk_level("ls; mkfs /dev/sda"), CommandRiskLevel::High);

    p.require_approval_for_medium_risk = true;
    p.block_high_risk_commands = false;
    assert!(p.validate_command_execution("ls", false).is_ok());
    let medium = p.validate_command_execution("curl https://x.test", false).unwrap_err();
    assert!(medium.contains("medium-risk"), "{medium}");
    assert!(p.validate_command_execution("curl https://x.test", true).is_ok());
    let high = p.validate_command_execution("mkfs /dev/sda", false).unwrap_err();
    assert!(high.contains("high-risk"), "{high}");
    assert!(p.validate_command_execution("mkfs /dev/sda", true).is_ok());

    p.block_high_risk_commands = true;
    let blocked = p.validate_command_execution("mkfs /dev/sda", true).unwrap_err();
    assert!(blocked.contains(POLICY_BLOCKED_MARKER), "{blocked}");

    let long = format!("zzz {}", "a".repeat(400));
    let err = p.validate_command_execution(&long, true).unwrap_err();
    assert!(err.len() < 300, "denial text must truncate the command: {}", err.len());
}

#[test]
fn always_forbidden_floor_covers_credential_stores_and_system_roots() {
    for forbidden in [
        "/home/u/.ssh/id_rsa",
        "/home/u/.gnupg/pubring.kbx",
        "/home/u/.aws/credentials",
        "/home/u/.kube/config",
        "/Users/u/Library/Keychains/login.keychain-db",
        "/home/u/.netrc",
        "/home/u/.git-credentials",
        "/home/u/.docker/config.json",
        "/home/u/.cargo/credentials.toml",
        "C:\\Users\\u\\AppData\\Roaming\\Microsoft\\Protect\\blob",
        "/etc/passwd",
        "/root/.bashrc",
        "/proc/self/environ",
        "/sys/kernel",
        "C:/Windows/System32",
        "/ETC/shadow",
    ] {
        assert!(
            SecurityPolicy::is_always_forbidden(Path::new(forbidden)),
            "floor must forbid {forbidden}"
        );
    }
    for allowed in [
        "/home/u/project/src/main.rs",
        "/tmp/work/file",
        "/private/var/folders/x",
        "/home/u/.docker/other.json",
        "/etcetera/file",
        "/home/u/notes.ssh.txt",
    ] {
        assert!(
            !SecurityPolicy::is_always_forbidden(Path::new(allowed)),
            "floor must not forbid {allowed}"
        );
    }
}

#[test]
fn floor_holds_with_policy_disabled_and_through_trusted_roots() {
    let off = SecurityPolicy {
        enabled: false,
        ..SecurityPolicy::default()
    };
    assert!(!off.is_resolved_path_allowed(Path::new("/home/u/.ssh/id_rsa")));
    assert!(!off.is_resolved_path_allowed(Path::new("/etc/passwd")));
    assert!(off.is_resolved_path_allowed(Path::new("/anywhere/else/file")));
    assert!(off.check_protected_path_literals("cat ~/.ssh/id_rsa").is_err());
    assert!(off.check_protected_path_literals("cat notes.txt").is_ok());

    let on = SecurityPolicy {
        enabled: true,
        trusted_roots: vec![TrustedRoot {
            path: "/home/u".into(),
            access: TrustedAccess::ReadWrite,
        }],
        ..SecurityPolicy::default()
    };
    assert!(on.is_within_trusted_root(Path::new("/home/u/project"), true));
    assert!(!on.is_within_trusted_root(Path::new("/home/u/.ssh/id_rsa"), false));
    assert!(!on.is_within_trusted_root(Path::new("/home/u/.aws/credentials"), true));
}

#[test]
fn trusted_root_access_tiers_are_pinned() {
    let p = SecurityPolicy {
        enabled: true,
        trusted_roots: vec![
            TrustedRoot { path: "/data/ro".into(), access: TrustedAccess::ReadOnly },
            TrustedRoot { path: "/data/rw".into(), access: TrustedAccess::ReadWrite },
        ],
        ..SecurityPolicy::default()
    };
    assert!(p.is_within_trusted_root(Path::new("/data/ro/a"), false));
    assert!(!p.is_within_trusted_root(Path::new("/data/ro/a"), true));
    assert!(p.is_within_trusted_root(Path::new("/data/rw/a"), true));
    assert!(!p.is_within_trusted_root(Path::new("/data/other/a"), false));
}

#[test]
fn hourly_rate_limiter_counts_acts_only_and_is_inert_when_disabled() {
    let p = SecurityPolicy {
        enabled: true,
        autonomy: AutonomyLevel::Supervised,
        max_actions_per_hour: 3,
        ..SecurityPolicy::default()
    };
    for _ in 0..10 {
        assert!(p.enforce_tool_operation(ToolOperation::Read, "read").is_ok());
    }
    assert_eq!(p.tracker.count(), 0, "reads never consume the budget");
    assert!(!p.is_rate_limited());
    for n in 1..=3 {
        assert!(p.enforce_tool_operation(ToolOperation::Act, "act").is_ok(), "act {n}");
    }
    assert!(p.is_rate_limited());
    let err = p.enforce_tool_operation(ToolOperation::Act, "act").unwrap_err();
    assert!(err.contains("Rate limit exceeded"), "{err}");
    assert!(err.contains("3 actions/hour"), "{err}");

    let off = SecurityPolicy {
        enabled: false,
        max_actions_per_hour: 1,
        ..SecurityPolicy::default()
    };
    for _ in 0..5 {
        assert!(off.enforce_tool_operation(ToolOperation::Act, "act").is_ok());
    }
    assert!(!off.is_rate_limited());
}

#[test]
fn read_only_tier_blocks_acts_before_the_budget() {
    let p = SecurityPolicy {
        enabled: true,
        autonomy: AutonomyLevel::ReadOnly,
        max_actions_per_hour: 100,
        ..SecurityPolicy::default()
    };
    assert!(!p.can_act());
    let err = p.enforce_tool_operation(ToolOperation::Act, "write").unwrap_err();
    assert!(err.contains(POLICY_BLOCKED_MARKER), "{err}");
    assert_eq!(p.tracker.count(), 0);
    assert!(p.enforce_write_tier("memory").is_err());
}

/// DESIRED (issue #7328 phase 0): a denial for a command the policy blocked must
/// not echo the command's argument text, since arguments carry secrets. Today the
/// allowlist denial truncates to 80 chars but still echoes the prefix. Expected to
/// fail until the tinysecurity module owns the denial text; flip when it does.
#[test]
#[should_panic(expected = "DESIRED-RED")]
fn desired_allowlist_denial_does_not_echo_command_arguments() {
    let mut p = policy(AutonomyLevel::Supervised);
    p.allowed_commands = vec!["ls".into()];
    let err = p
        .validate_command_execution("psql postgres://user:hunter2@db/prod", true)
        .unwrap_err();
    assert!(
        !err.contains("hunter2"),
        "DESIRED-RED: denial echoes the secret-bearing command prefix: {err}"
    );
}
