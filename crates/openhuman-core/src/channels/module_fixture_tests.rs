//! A linked TinyChannels module fixture for host-side unit tests.
//!
//! These tests exercise the real owning module over TinyBus without downloading
//! a release artifact or reaching a provider service. The fixture broker and
//! module stay on a dedicated process-lifetime runtime because TinyBus module
//! tasks must outlive the short Tokio runtimes used by individual tests.

use crate::modules::client::ModuleClient;
use std::sync::{Mutex, OnceLock};
use tinybus::Proxy;
use tinychannels_bus::methods;

static FIXTURE_PROXY: OnceLock<Proxy> = OnceLock::new();
static FIXTURE_INIT: Mutex<()> = Mutex::new(());

/// Return a client for the compiled, linked TinyChannels module fixture.
pub(crate) async fn client() -> ModuleClient {
    if let Some(proxy) = FIXTURE_PROXY.get() {
        return ModuleClient::fixture(proxy.clone());
    }

    let _guard = FIXTURE_INIT
        .lock()
        .expect("the TinyChannels fixture initialization lock is not poisoned");
    if let Some(proxy) = FIXTURE_PROXY.get() {
        return ModuleClient::fixture(proxy.clone());
    }

    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("tinychannels-test-module".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .thread_name("tinychannels-test-bus")
                .build()
                .expect("the TinyChannels test runtime should start");
            runtime.block_on(async move {
                let transport = tinybus::transport::memory::MemoryBus::new();
                let broker = tinybus::broker::Broker::new();
                broker.spawn(transport.clone());
                let connection = tinybus::Connection::connect(
                    transport
                        .connect()
                        .await
                        .expect("the TinyChannels fixture broker should accept a connection"),
                )
                .await
                .expect("the TinyChannels fixture host should connect to its broker");
                let host = tinybus::module::ModuleHost::new(broker);
                host.attach_linked_with_config(
                    tinychannels_module::linked_module()
                        .expect("the linked TinyChannels module should be available"),
                    serde_json::json!({}),
                )
                .expect("the linked TinyChannels module should be admitted");
                let proxy = connection
                    .proxy(
                        tinychannels_module::BUS_NAME,
                        tinychannels_module::OBJECT_PATH,
                        tinychannels_module::BUS_NAME,
                    )
                    .expect("the linked TinyChannels proxy should be valid");
                let _ = ready_tx.send(proxy);

                // Keep the connection and module host alive with the broker
                // until process exit. Aborting this task would fault the
                // attached module and invalidate proxies held by other tests.
                let (_connection, _host) = (connection, host);
                std::future::pending::<()>().await;
            });
        })
        .expect("the TinyChannels fixture thread should start");

    let proxy = ready_rx
        .recv()
        .expect("the linked TinyChannels module should finish setup");
    let _ = FIXTURE_PROXY.set(proxy.clone());
    ModuleClient::fixture(FIXTURE_PROXY.get().unwrap().clone())
}

#[tokio::test]
async fn linked_module_prepares_runtime_limits_over_confidential_bus_call() {
    use tinychannels_bus::module_control::{ChannelControlRequest, ChannelControlResult};

    let response: ChannelControlResult = client()
        .await
        .call_confidential(
            "tinychannels",
            methods::PREPARE_CHANNEL_CONTROL,
            (ChannelControlRequest::RuntimeLimits {
                timeout_secs: 1,
                listener_count: 100,
            },),
        )
        .await
        .expect("the attached linked module should answer a confidential call");

    assert_eq!(
        response,
        ChannelControlResult::RuntimeLimits {
            timeout_secs: 30,
            max_in_flight: 64,
        }
    );
}

#[tokio::test]
async fn linked_module_prepares_legacy_inbound_facts_over_confidential_bus_call() {
    use tinychannels_bus::module_inbound::{
        ChannelInboundSource, PrepareChannelInboundRequest, PreparedChannelInbound,
    };
    use tinychannels_bus::ChannelMessage;

    let message = ChannelMessage {
        id: "message-1".into(),
        channel: "discord".into(),
        sender: "sender/a".into(),
        reply_target: "room#b".into(),
        content: "fixture content".into(),
        timestamp: 42,
        thread_ts: Some("thread/c".into()),
        sender_name: Some("Display".into()),
    };
    let response: PreparedChannelInbound = client()
        .await
        .call_confidential(
            "tinychannels",
            methods::PREPARE_CHANNEL_INBOUND,
            (PrepareChannelInboundRequest {
                source: ChannelInboundSource::Legacy {
                    message,
                    envelope: None,
                },
                namespace: "openhuman".into(),
                session_policy: Default::default(),
            },),
        )
        .await
        .expect("the attached linked module should prepare inbound facts");

    assert_eq!(
        response.thread_id,
        "channel:discord/sender%2Fa/room%23b#thread:thread%2Fc"
    );
    assert_eq!(response.client_id, "inbound:discord:sender/a");
    assert_eq!(
        response.history_key,
        "discord_sender/a_room#b_thread:thread/c"
    );
    assert_eq!(
        response.session_key,
        "openhuman:discord:default:unknown:room#b:thread/c"
    );
    assert_eq!(response.message.timestamp, 42);
    assert_eq!(response.message.sender_name.as_deref(), Some("Display"));
    assert_eq!(response.envelope.text, "fixture content");
}
