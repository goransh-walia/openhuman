//! Tests for the host half of privacy enforcement: which class a descriptor
//! belongs to, what is asked of the module, and how its answer is rendered. What
//! the module decides for each mode is pinned in the module's own tests.

use super::*;
use crate::config::PrivacyMode;
use crate::security::live_policy::test_privacy_scope;

fn external_composio() -> EgressDescriptor {
    EgressDescriptor::composio("GITHUB_CREATE_ISSUE")
}

// ── data_class: the host's knowledge of which routes carry user content ─────

#[test]
fn control_plane_exempts_non_tool_namespace() {
    // A control-plane call re-homed onto an integration descriptor: anything not
    // under /agent-integrations/ is exempt.
    for path in [
        "/teams/me/usage",
        "/payments/stripe/currentPlan",
        "/auth/refresh",
    ] {
        assert_eq!(
            data_class(&EgressDescriptor::integration(path)),
            PrivacyClass::ControlPlane,
            "{path} must be treated as control-plane"
        );
    }
}

#[test]
fn control_plane_exempts_composio_management_and_pricing() {
    for path in [
        "/agent-integrations/pricing",
        "/agent-integrations/composio/connections",
        // Per-connection delete rides the same `connections` head.
        "/agent-integrations/composio/connections/conn_123",
        "/agent-integrations/composio/authorize",
        "/agent-integrations/composio/tools",
        "/agent-integrations/composio/toolkits",
    ] {
        assert_eq!(
            data_class(&EgressDescriptor::integration(path)),
            PrivacyClass::ControlPlane,
            "{path} must be exempt (control-plane)"
        );
    }
}

#[test]
fn control_plane_does_not_exempt_user_data_paths() {
    // composio/execute ships tool arguments; composio/triggers[/available] POST
    // user-supplied slug/connectionId/triggerConfig; github/repos reveals
    // user-adjacent data; the other tool namespaces ship queries and content.
    // An unknown composio sub-route defaults to user data (fail-closed).
    for path in [
        "/agent-integrations/composio/execute",
        "/agent-integrations/composio/triggers",
        "/agent-integrations/composio/triggers/available",
        "/agent-integrations/composio/github/repos",
        "/agent-integrations/composio/some-future-write",
        "/agent-integrations/parallel/research",
        "/agent-integrations/tinyfish/fetch",
        "/agent-integrations/file-storage/files",
        "/agent-integrations/google-places/search",
    ] {
        assert_ne!(
            data_class(&EgressDescriptor::integration(path)),
            PrivacyClass::ControlPlane,
            "{path} must NOT be exempt (ships user data)"
        );
    }
}

#[test]
fn control_plane_only_applies_to_integration_reason() {
    // A network fetch whose host happens to spell a backend path is still a
    // user-data transfer, never control-plane.
    let net = EgressDescriptor::network_fetch("agent-integrations");
    assert_eq!(net.reason, EgressReason::NetworkFetch);
    assert_eq!(data_class(&net), PrivacyClass::UserContent);
}

#[test]
fn a_metadata_only_transfer_is_metadata_and_anything_more_is_content() {
    let routing = EgressDescriptor::integration("/agent-integrations/parallel/usage");
    assert_eq!(routing.data_kinds, vec![DataKind::Metadata]);
    assert_eq!(data_class(&routing), PrivacyClass::Metadata);
    let upload = routing.with_data_kind(DataKind::FileContent);
    assert_eq!(data_class(&upload), PrivacyClass::UserContent);
}

// ── Standard asks nothing; refusal rendering ───────────────────────────────

#[tokio::test]
async fn standard_mode_never_reaches_the_module() {
    // No port is installed and no module runtime exists: a question would fail
    // closed. Standard registers no restriction, so none is asked.
    assert!(egress_blocks_message(PrivacyMode::Standard, &external_composio())
        .await
        .is_none());
    let _mode = test_privacy_scope(PrivacyMode::Standard);
    enforce_egress(&external_composio())
        .await
        .expect("Standard mode must allow");
}

#[test]
fn refusal_messages_name_the_service_and_the_mode() {
    let desc = external_composio();
    let local = block_message(PrivacyMode::LocalOnly, &desc, Outcome::Refused);
    assert!(local.contains("Local-only privacy mode is active"), "{local}");
    assert!(local.contains("GITHUB_CREATE_ISSUE"), "{local}");
    let sensitive = block_message(PrivacyMode::Sensitive, &desc, Outcome::Refused);
    assert!(sensitive.contains("Sensitive privacy mode is active"), "{sensitive}");
    let unavailable = block_message(PrivacyMode::LocalOnly, &desc, Outcome::Unavailable);
    assert!(unavailable.contains("could not confirm"), "{unavailable}");
}

#[cfg(feature = "security-module")]
mod with_module {
    use super::*;
    use crate::modules::security::gap::stub::{Asked, StubPort};
    use crate::modules::security::gap::install_port;
    use tinysecurity_bus::{
        PrivacyDataClass2, PrivacyMode as BusMode, PrivacyRiskLevel2, Verdict,
    };

    fn refusing() -> Arc<StubPort> {
        Arc::new(StubPort::allowing().privacy(|_| Verdict::Deny {
            reason: tinysecurity_bus::DenialReason::Privacy,
        }))
    }

    use std::sync::Arc;

    #[tokio::test]
    async fn a_restrictive_mode_asks_the_module_with_the_descriptor_facts() {
        let stub = refusing();
        let _port = install_port(stub.clone());
        let desc = external_composio().with_risk(IdentificationRisk::Low, vec!["email".into()]);
        let message = egress_blocks_message(PrivacyMode::Sensitive, &desc)
            .await
            .expect("the module refused");
        assert!(message.contains("Sensitive privacy mode is active"), "{message}");
        let asked = stub.asked();
        let [Asked::Privacy(request)] = asked.as_slice() else {
            panic!("exactly one privacy question, got {asked:?}");
        };
        assert_eq!(request.privacy, BusMode::Sensitive);
        assert!(request.external);
        assert_eq!(request.class, PrivacyDataClass2::UserContent);
        assert_eq!(request.destination_label, "composio");
        let risk = request.risk.as_ref().expect("assessed risk is forwarded");
        assert_eq!(risk.level, PrivacyRiskLevel2::Low);
        assert_eq!(risk.categories, vec!["email".to_string()]);
    }

    #[tokio::test]
    async fn an_unassessed_payload_is_forwarded_as_no_risk() {
        let stub = refusing();
        let _port = install_port(stub.clone());
        let _ = egress_blocks_message(PrivacyMode::Sensitive, &external_composio()).await;
        let asked = stub.asked();
        let [Asked::Privacy(request)] = asked.as_slice() else {
            panic!("exactly one privacy question");
        };
        assert!(request.risk.is_none(), "Unknown means not assessed");
    }

    #[tokio::test]
    async fn control_plane_and_local_runtime_facts_are_forwarded() {
        let stub = Arc::new(StubPort::allowing());
        let _port = install_port(stub.clone());
        let pricing = EgressDescriptor::integration("/agent-integrations/pricing");
        assert!(egress_blocks_message(PrivacyMode::LocalOnly, &pricing).await.is_none());
        let local = EgressDescriptor::inference("ollama", "llama3", false);
        assert!(egress_blocks_message(PrivacyMode::LocalOnly, &local).await.is_none());
        let asked = stub.asked();
        let classes: Vec<_> = asked
            .iter()
            .map(|asked| match asked {
                Asked::Privacy(request) => (request.class, request.external),
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(
            classes,
            vec![
                (PrivacyDataClass2::ControlPlane, true),
                (PrivacyDataClass2::UserContent, false)
            ]
        );
    }

    #[tokio::test]
    async fn an_unavailable_module_refuses_under_a_restrictive_mode() {
        let _port = install_port(Arc::new(StubPort::unavailable()));
        let message = egress_blocks_message(PrivacyMode::LocalOnly, &external_composio())
            .await
            .expect("fails closed");
        assert!(message.contains("could not confirm"), "{message}");
        let _mode = test_privacy_scope(PrivacyMode::LocalOnly);
        assert!(enforce_egress(&external_composio()).await.is_err());
    }

    #[tokio::test]
    async fn the_tool_block_carries_the_policy_marker_and_clears_when_allowed() {
        let mut desc = EgressDescriptor::network_fetch("api.example.com");
        desc = desc.with_data_kind(DataKind::ToolArguments);
        {
            let _port = install_port(refusing());
            let _mode = test_privacy_scope(PrivacyMode::LocalOnly);
            let message = local_only_tool_block(&desc).await.expect("blocked");
            assert!(message.starts_with(POLICY_BLOCKED_MARKER), "{message}");
            assert!(message.contains("api.example.com"), "{message}");
            assert!(local_only_tool_block_blocking(&desc).is_some());
        }
        let _port = install_port(Arc::new(StubPort::allowing()));
        let _mode = test_privacy_scope(PrivacyMode::LocalOnly);
        assert!(local_only_tool_block(&desc).await.is_none());
        assert!(local_only_tool_block_blocking(&desc).is_none());
    }
}
