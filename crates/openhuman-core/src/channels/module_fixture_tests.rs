//! A linked TinyChannels module fixture for host-side unit tests.
//!
//! These tests exercise the real owning module over TinyBus without downloading
//! a release artifact or reaching a provider service. The module is attached
//! through the same seam local development uses: the factory is registered with
//! [`register_linked`] and `[modules] source = "linked"` makes
//! `ops::ensure_loaded` attach it to the process-wide module host, which lives
//! on a process-lifetime runtime, so it outlives the short Tokio runtimes of
//! individual tests.

use crate::modules::client::ModuleClient;
use tinychannels_bus::methods;

/// Return a client for the compiled, linked TinyChannels module fixture.
///
/// BLOCKED on the tinybus epic: `tinychannels-module` v0.1.29 pins `tinybus =0.1.4`
/// while the workspace tinybus is 0.1.3, so the module cannot be a dev-dependency
/// and the fixture cannot be linked. Tests that need it fail with this message
/// until the dependency is restored.
pub(crate) async fn client() -> ModuleClient {
    panic!("tinychannels-module fixture unavailable: blocked on tinybus 0.1.4 (tinybus epic)")
}

#[tokio::test]
#[ignore = "blocked on the linked tinychannels-module fixture (tinybus epic, #7739)"]
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
#[ignore = "blocked on the linked tinychannels-module fixture (tinybus epic, #7739)"]
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
