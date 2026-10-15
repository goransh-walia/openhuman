//! Property-based tests for the path check's adversarial-input surface
//! (plan.md §6.3 — proptest was previously absent). Paths arrive as fully
//! untrusted strings from the LLM/user, so the central property is "never
//! panics on any input". The command classifier these once also covered is the
//! TinySecurity module's now, and is fuzzed there; the host's translation of a
//! policy into the module's settings, and of the module's answer back, is
//! covered here.

use super::types::SecurityPolicy;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// The string-level path check must never panic on arbitrary paths
    /// (traversal, NUL bytes, tilde, URL-encoding, unicode).
    #[test]
    fn path_string_check_never_panics(path in ".*") {
        let p = SecurityPolicy::default();
        let _ = p.is_path_string_allowed(&path);
    }
}

#[cfg(feature = "security-module")]
mod command_translation {
    use super::super::native_commands::{
        project, CommandFlow, CommandQuery, CommandStage, Refusal,
    };
    use super::super::types::{AutonomyLevel, SecurityPolicy};
    use proptest::prelude::*;
    use tinysecurity_bus::{CommandClass, CommandDecision, CommandDenial, CommandGate, CommandRisk};

    fn class() -> impl Strategy<Value = CommandClass> {
        prop_oneof![
            Just(CommandClass::Read),
            Just(CommandClass::Write),
            Just(CommandClass::Network),
            Just(CommandClass::Install),
            Just(CommandClass::Destructive),
        ]
    }

    fn gate() -> impl Strategy<Value = CommandGate> {
        prop_oneof![
            Just(CommandGate::Allow),
            Just(CommandGate::Prompt),
            Just(CommandGate::Block)
        ]
    }

    fn denial() -> impl Strategy<Value = Option<CommandDenial>> {
        prop_oneof![
            Just(None),
            Just(Some(CommandDenial::Protected)),
            Just(Some(CommandDenial::ReadOnly)),
            Just(Some(CommandDenial::Command)),
            Just(Some(CommandDenial::HighRisk)),
            Just(Some(CommandDenial::RateLimit)),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        /// Projecting any module answer never panics; a denial or a blocked
        /// real command is always a refusal, and no refusal message carries the
        /// command text.
        #[test]
        fn projection_refuses_every_denial_and_never_echoes_the_command(
            class in class(),
            gate in gate(),
            denial in denial(),
            scheduled in any::<bool>(),
            class_only in any::<bool>(),
            command in "[ -~]{8,40}",
        ) {
            let query = CommandQuery {
                command: &command,
                declared: None,
                flow: if scheduled { CommandFlow::Scheduled } else { CommandFlow::Harness },
                stage: CommandStage::Execute,
            };
            let answer = CommandDecision { class, risk: CommandRisk::Low, gate, denial };
            let verdict = project(&query, class_only, Ok(answer));
            if denial.is_some() || (gate == CommandGate::Block && !class_only) {
                prop_assert!(verdict.is_err());
            }
            if let Err(refusal) = verdict {
                prop_assert!(!refusal.to_string().contains(&command));
            }
        }

        /// A module that fails to answer always denies.
        #[test]
        fn an_unanswered_question_is_always_refused(command in ".*", class_only in any::<bool>()) {
            let query = CommandQuery {
                command: &command,
                declared: None,
                flow: CommandFlow::Harness,
                stage: CommandStage::Authorize,
            };
            prop_assert_eq!(project(&query, class_only, Err(command.clone())), Err(Refusal::Unavailable));
        }

        /// Translating a policy never panics, whatever its allowlist holds, and
        /// the authorize stage never carries a spendable budget.
        #[test]
        fn policy_translation_never_panics(
            allowed in proptest::collection::vec(".*", 0..8),
            budget in any::<u32>(),
            enabled in any::<bool>(),
        ) {
            let policy = SecurityPolicy {
                enabled,
                autonomy: AutonomyLevel::Supervised,
                allowed_commands: allowed,
                max_actions_per_hour: budget,
                ..SecurityPolicy::default()
            };
            let authorize = policy
                .native_command_policy(CommandFlow::Harness, CommandStage::Authorize)
                .expect("an absolute or resolvable action dir translates");
            prop_assert_eq!(authorize.max_actions_per_hour, u32::MAX);
            let execute = policy
                .native_command_policy(CommandFlow::Scheduled, CommandStage::Execute)
                .expect("translates");
            prop_assert_eq!(execute.max_actions_per_hour, budget);
        }
    }
}
