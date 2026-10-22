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

// This linked module claims TinyComputer's real interface name through
// ModuleHost, which gives its calls an authenticated module sender. A plain
// Connection that merely requests the same name does not have that identity.
struct AuthenticatedComputerPeer(Connection);

#[async_trait::async_trait]
impl tinybus::Interface for AuthenticatedComputerPeer {
    fn name(&self) -> tinybus::InterfaceName {
        tinycomputer_bus::names::INTERFACE.try_into().unwrap()
    }

    fn members(&self) -> Vec<tinybus::MemberName> {
        ["Authorize", "Consume", "Validate"]
            .into_iter()
            .map(|member| member.try_into().unwrap())
            .collect()
    }

    async fn call(
        &self,
        member: &tinybus::MemberName,
        args: serde_json::Value,
    ) -> tinybus::Result<serde_json::Value> {
        let callback = self.0.proxy(
            contract::INTERFACE,
            contract::OBJECT_PATH,
            contract::INTERFACE,
        )?;
        match member.as_str() {
            "Authorize" => {
                let (target,): (BrowserEgressTarget,) = serde_json::from_value(args)
                    .map_err(|error| tinybus::Error::failed(error.to_string()))?;
                let grant: contract::BrowserEgressGrant =
                    callback.call(methods::AUTHORIZE, (target,)).await?;
                serde_json::to_value(grant)
                    .map_err(|error| tinybus::Error::failed(error.to_string()))
            }
            "Consume" => {
                let (request,): (BrowserEgressConsumeRequest,) = serde_json::from_value(args)
                    .map_err(|error| tinybus::Error::failed(error.to_string()))?;
                let decision: BrowserEgressDecision =
                    callback.call(methods::CONSUME, (request,)).await?;
                serde_json::to_value(decision)
                    .map_err(|error| tinybus::Error::failed(error.to_string()))
            }
            "Validate" => {
                let (request,): (BrowserEgressValidateRequest,) = serde_json::from_value(args)
                    .map_err(|error| tinybus::Error::failed(error.to_string()))?;
                let decision: BrowserEgressDecision =
                    callback.call(methods::VALIDATE, (request,)).await?;
                serde_json::to_value(decision)
                    .map_err(|error| tinybus::Error::failed(error.to_string()))
            }
            _ => Err(tinybus::Error::failed("unknown test peer method")),
        }
    }
}

async fn authenticated_computer_setup(connection: Connection) -> tinybus::Result<()> {
    connection
        .serve_at(
            tinycomputer_bus::names::OBJECT_PATH.try_into().unwrap(),
            AuthenticatedComputerPeer(connection.clone()),
        )
        .await?;
    connection
        .request_name(tinycomputer_bus::names::INTERFACE)
        .await?;
    Ok(())
}

mod authenticated_computer_export {
    tinybus_module::module_export_static! {
        setup = super::authenticated_computer_setup,
        worker_threads = 1,
        provides = ["ai.tinyhumans.tinycomputer.Desktop"],
        methods = ["Authorize", "Consume", "Validate"],
        signals = [],
        requires = [],
        optional = [],
        lazy = false,
    }
}

async fn callback_bus() -> (
    MemoryBus,
    Connection,
    tokio::task::JoinHandle<tinybus::Result<()>>,
) {
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
        .proxy(
            contract::INTERFACE,
            contract::OBJECT_PATH,
            contract::INTERFACE,
        )
        .unwrap();
    let result: tinybus::Result<contract::BrowserEgressGrant> = proxy
        .call(
            methods::AUTHORIZE,
            (target("host-policy", 1, "8.8.8.8", 443),),
        )
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
        .proxy(
            contract::INTERFACE,
            contract::OBJECT_PATH,
            contract::INTERFACE,
        )
        .unwrap();
    let result: tinybus::Result<contract::BrowserEgressGrant> = proxy
        .call(
            methods::AUTHORIZE,
            (target("unregistered", 1, "8.8.8.8", 443),),
        )
        .await;
    assert!(result.is_err(), "unknown host policy IDs must be refused");
}

/// Exercises the authenticated callback against a real TinySecurity cdylib.
/// The fixture is supplied by `scripts/ci/security-browser-egress-fixture.sh`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires OPENHUMAN_TEST_SECURITY_MODULE pointing to the locally built TinySecurity module"]
async fn browser_egress_callbacks_use_tinysecurity_one_use_grants_and_leases() {
    use crate::modules::security::native_tests::native_fixture_configured;

    let fixture = native_fixture_configured(false, |_| {}).await;
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

    let host_callback = fixture
        .client
        .proxy(
            contract::INTERFACE,
            contract::OBJECT_PATH,
            contract::INTERFACE,
        )
        .unwrap();
    assert!(host_callback.is_available().await.unwrap());
    fixture
        .host
        .attach_linked_with_config(
            authenticated_computer_export::linked_module().unwrap(),
            serde_json::json!({}),
        )
        .unwrap();
    let attacker = Connection::connect(fixture.transport.connect().await.unwrap())
        .await
        .unwrap();
    attacker
        .request_name("ai.tinyhumans.tinycomputer.Desktop")
        .await
        .unwrap_err();
    let attacker_callback = attacker
        .proxy(
            contract::INTERFACE,
            contract::OBJECT_PATH,
            contract::INTERFACE,
        )
        .unwrap();
    let spoofed: tinybus::Result<contract::BrowserEgressGrant> = attacker_callback
        .call(
            methods::AUTHORIZE,
            (target(&policy_id.0, info.generation, "8.8.8.8", 443),),
        )
        .await;
    assert!(
        spoofed.is_err(),
        "a plain peer cannot impersonate TinyComputer"
    );
    let proxy = fixture
        .client
        .proxy(
            tinycomputer_bus::names::INTERFACE,
            tinycomputer_bus::names::OBJECT_PATH,
            tinycomputer_bus::names::INTERFACE,
        )
        .unwrap();

    let direct = crate::modules::security::egress::authorize_browser_dial(target(
        &policy_id.0,
        info.generation,
        "8.8.8.8",
        443,
    ))
    .await
    .expect("direct host adapter authorization should expose authority decoding errors");
    assert!(!direct.authorization.is_empty());
    let target = target(&policy_id.0, info.generation, "8.8.8.8", 443);
    let grant: contract::BrowserEgressGrant = proxy
        .call("Authorize", (target.clone(),))
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
