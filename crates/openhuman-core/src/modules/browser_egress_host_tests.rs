use super::*;
use tinybus::{broker::Broker, transport::memory::MemoryBus, Connection};
use tinycomputer_bus::browser_egress::{
    methods, BrowserEgressConsumeRequest, BrowserEgressTarget, BrowserEgressValidateRequest,
};

fn target(policy_id: &str, generation: u64, hostname: &str, port: u16) -> BrowserEgressTarget {
    BrowserEgressTarget {
        policy_id: policy_id.to_owned(),
        expected_generation: generation,
        session_id: "browser-session-test-7f9a".into(),
        scheme: "https".into(),
        hostname: hostname.to_owned(),
        port,
    }
}

async fn callback_bus() -> (MemoryBus, Connection, tokio::task::JoinHandle<tinybus::Result<()>>) {
    let transport = MemoryBus::new();
    let broker = Broker::new();
    let task = broker.spawn(transport.clone());
    let host = Connection::connect(transport.connect().await.unwrap())
        .await
        .unwrap();
    install(&host).await.unwrap();
    (transport, host, task)
}

#[tokio::test]
async fn browser_egress_callback_rejects_unauthenticated_peers() {
    let (transport, _host, _task) = callback_bus().await;
    let caller = Connection::connect(transport.connect().await.unwrap())
        .await
        .unwrap();
    let proxy = caller
        .proxy(contract::INTERFACE, contract::OBJECT_PATH, contract::INTERFACE)
        .unwrap();
    let result: tinybus::Result<contract::BrowserEgressGrant> = proxy
        .call(methods::AUTHORIZE, (target("host-policy", 1, "8.8.8.8", 443),))
        .await;
    assert!(result.is_err(), "an unclaimed bus peer must be refused");
}

#[tokio::test]
async fn browser_egress_callback_fails_closed_without_a_host_registered_scope() {
    let (transport, _host, _task) = callback_bus().await;
    let caller = Connection::connect(transport.connect().await.unwrap())
        .await
        .unwrap();
    caller
        .request_name(tinycomputer_bus::names::INTERFACE)
        .await
        .unwrap();
    let proxy = caller
        .proxy(contract::INTERFACE, contract::OBJECT_PATH, contract::INTERFACE)
        .unwrap();
    let result: tinybus::Result<contract::BrowserEgressGrant> = proxy
        .call(methods::AUTHORIZE, (target("unregistered", 1, "8.8.8.8", 443),))
        .await;
    assert!(result.is_err(), "unknown host policy IDs must be refused");
}

/// Exercises the authenticated callback against a real TinySecurity cdylib.
/// The fixture is supplied by `scripts/ci/security-browser-egress-fixture.sh`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires OPENHUMAN_TEST_SECURITY_MODULE pointing to the locally built TinySecurity module"]
async fn browser_egress_callbacks_use_tinysecurity_one_use_grants_and_leases() {
    use crate::modules::security::native_tests::native_fixture;

    let fixture = native_fixture().await;
    install(&fixture.client).await.unwrap();
    let owner = tinysecurity_bus::SubjectScope {
        user: "native-test-user".into(),
        workspace: "native-test-workspace".into(),
        agent: "native-test-agent".into(),
    };
    let policy = tinysecurity_bus::EgressPolicy {
        owner,
        ceiling: tinysecurity_bus::AccessTier::Full,
        privacy: tinysecurity_bus::PrivacyMode::Standard,
        destinations: Vec::new(),
        data_categories: vec![tinysecurity_bus::DataCategory::OpaqueBrowserTraffic],
        local_capabilities: Vec::new(),
        validity_ms: 30_000,
        open_public: true,
        preflight_transports: vec![tinysecurity_bus::EgressTransport::Browser],
    };
    let policy_id: tinysecurity_bus::EgressPolicyId = fixture
        .proxy
        .call_confidential(
            tinysecurity_bus::names::methods::REGISTER_EGRESS_POLICY,
            (policy,),
        )
        .await
        .expect("trusted host registers browser policy");
    let info: tinysecurity_bus::PolicyInfo = fixture
        .proxy
        .call(tinysecurity_bus::names::methods::POLICY_INFO, ())
        .await
        .expect("read actual module generation");
    crate::modules::security::egress::remember_browser_dial_binding_for_test(
        policy_id.clone(),
        info.generation,
        fixture.proxy.clone(),
    );

    let caller = Connection::connect(fixture.transport.connect().await.unwrap())
        .await
        .unwrap();
    caller
        .request_name(tinycomputer_bus::names::INTERFACE)
        .await
        .unwrap();
    let proxy = caller
        .proxy(contract::INTERFACE, contract::OBJECT_PATH, contract::INTERFACE)
        .unwrap();

    let target = target(&policy_id.0, info.generation, "8.8.8.8", 443);
    let grant: contract::BrowserEgressGrant = proxy
        .call(methods::AUTHORIZE, (target.clone(),))
        .await
        .expect("TinySecurity authorizes a public IP without a network dial");
    assert_eq!(grant.generation, info.generation);
    assert_eq!(grant.addresses, ["8.8.8.8:443"]);
    assert!(!grant.authorization.is_empty());

    let consumed: BrowserEgressDecision = proxy
        .call(
            methods::CONSUME,
            (BrowserEgressConsumeRequest {
                target: target.clone(),
                authorization: grant.authorization.clone(),
                address: grant.addresses[0].clone(),
            },),
        )
        .await
        .expect("TinySecurity consumes the exact one-use grant");
    assert!(consumed.allowed);

    let valid: BrowserEgressDecision = proxy
        .call(
            methods::VALIDATE,
            (BrowserEgressValidateRequest {
                authorization: grant.authorization.clone(),
            },),
        )
        .await
        .expect("TinySecurity validates the live tunnel lease");
    assert!(valid.allowed);

    let second_consume: BrowserEgressDecision = proxy
        .call(
            methods::CONSUME,
            (BrowserEgressConsumeRequest {
                target,
                authorization: grant.authorization,
                address: grant.addresses[0].clone(),
            },),
        )
        .await
        .expect("replay is a denial, not a transport error");
    assert!(!second_consume.allowed, "one-use grants cannot be replayed");
}
