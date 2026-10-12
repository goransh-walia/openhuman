//! Identity-scoped implementation of TinyChannels' 14 host callbacks.

use super::delivery_store::ChannelDeliveryStore;
use crate::channels::context::ChannelRuntimeContext;
use crate::channels::runtime::dispatch::RuntimeChannelMessage;
use crate::config::Config;
use std::sync::{Arc, OnceLock, Weak};
use tinybus::{Error, Result};
use tinychannels_bus::{
    ChannelBackendRequest, ChannelBackendResult, DeliveryQueueEntry, DeliveryTerminalResult,
    HostEventPublication, HostTranscriptionRequest, HostTranscriptionResult,
};
use tokio::sync::mpsc;

const MAX_CALLBACK_BYTES: usize = 128 * 1024;
const MAX_TRANSCRIPTION_BYTES: usize = 12 * 1024 * 1024;
const MAX_CONTROLLER_BYTES: usize = 1024 * 1024;

/// Callbacks capture one immutable host config and a weak runtime context.
/// The weak reference avoids a cycle from runtime context -> module channel ->
/// lease -> TinyBus connection -> callback -> runtime context.
pub(super) struct ChannelHostCallbacks {
    config: Arc<Config>,
    core_context: Option<Arc<crate::core::runtime::CoreContext>>,
    tenant: crate::core::runtime::Tenant,
    cancellation: tokio_util::sync::CancellationToken,
    context: Arc<OnceLock<Weak<ChannelRuntimeContext>>>,
    dispatch: mpsc::Sender<RuntimeChannelMessage>,
    delivery: ChannelDeliveryStore,
}

impl ChannelHostCallbacks {
    pub(super) fn new(
        config: Arc<Config>,
        context: Arc<OnceLock<Weak<ChannelRuntimeContext>>>,
        dispatch: mpsc::Sender<RuntimeChannelMessage>,
    ) -> Self {
        let delivery = ChannelDeliveryStore::for_workspace(&config.workspace_dir);
        Self {
            config,
            core_context: crate::core::runtime::CoreContext::current(),
            tenant: crate::core::runtime::tenant::current_tenant_or_isolated(
                "channel_host_callback",
            ),
            cancellation: crate::channels::runtime::session::channel_session(),
            context,
            dispatch,
            delivery,
        }
    }

    fn validate_callback_size<T: serde::Serialize>(value: &T) -> Result<()> {
        if serde_json::to_vec(value).map_or(true, |bytes| bytes.len() > MAX_CALLBACK_BYTES) {
            return Err(Error::failed("host callback payload exceeds its limit"));
        }
        Ok(())
    }

    fn context(&self) -> Result<Arc<ChannelRuntimeContext>> {
        self.context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::failed("channel runtime is stopping"))
    }
}

#[tinybus::interface(name = "ai.tinyhumans.tinychannels.ChannelsHost")]
impl ChannelHostCallbacks {
    async fn deliver_inbound(&self, channel: String, message: serde_json::Value) -> Result<()> {
        Self::validate_callback_size(&message)?;
        if let Ok(delivery) = serde_json::from_value::<
            tinychannels_bus::module_relay::RelayInboundDelivery,
        >(message.clone())
        {
            if delivery.envelope.channel.id != channel {
                return Err(Error::failed(
                    "relay callback channel does not match envelope",
                ));
            }
            self.dispatch
                .send(RuntimeChannelMessage::from_authenticated_relay(delivery))
                .await
                .map_err(|_| Error::failed("channel dispatch is closed"))?;
            return Ok(());
        }

        let message: tinychannels_bus::ChannelMessage = serde_json::from_value(message)
            .map_err(|_| Error::failed("invalid inbound message"))?;
        if message.channel != channel {
            return Err(Error::failed("callback channel does not match message"));
        }
        self.dispatch
            .send(RuntimeChannelMessage::from(message))
            .await
            .map_err(|_| Error::failed("channel dispatch is closed"))
    }

    async fn report_status(
        &self,
        channel: String,
        state: String,
        _detail: Option<String>,
    ) -> Result<()> {
        if channel.is_empty() || channel.len() > 256 || state.len() > 128 {
            return Err(Error::failed("channel status is invalid"));
        }
        match state.as_str() {
            "connected" => crate::core::bus::BUS
                .publish(crate::core::events::DomainEvent::ChannelConnected { channel }),
            "disconnected" | "stopped" => crate::core::bus::BUS.publish(
                crate::core::events::DomainEvent::ChannelDisconnected {
                    channel,
                    reason: state,
                },
            ),
            // `connecting`, `reconnecting`, and `unhealthy` are health facts,
            // not user-facing lifecycle events.
            _ => {}
        }
        Ok(())
    }

    async fn persist_allowed_identity(&self, channel: String, identity: String) -> Result<()> {
        crate::channels::host::ConfigAllowlistStore
            .persist_allowed_identity(&channel, &identity)
            .await
            .map_err(|_| Error::failed("channel identity persistence failed"))
    }

    async fn save_pending_delivery(&self, entry: DeliveryQueueEntry) -> Result<()> {
        Self::validate_callback_size(&entry)?;
        self.delivery
            .save_pending(entry)
            .await
            .map_err(|_| Error::failed("delivery persistence failed"))
    }

    async fn load_pending_delivery(&self, id: String) -> Result<Option<DeliveryQueueEntry>> {
        self.delivery
            .load_pending(id)
            .await
            .map_err(|_| Error::failed("delivery lookup failed"))
    }

    async fn load_pending_deliveries(&self) -> Result<Vec<DeliveryQueueEntry>> {
        self.delivery
            .load_pending_all()
            .await
            .map_err(|_| Error::failed("delivery lookup failed"))
    }

    async fn remove_pending_delivery(&self, id: String) -> Result<()> {
        self.delivery
            .remove_pending(id)
            .await
            .map_err(|_| Error::failed("delivery removal failed"))
    }

    async fn save_failed_delivery(&self, entry: DeliveryQueueEntry) -> Result<()> {
        Self::validate_callback_size(&entry)?;
        self.delivery
            .save_failed(entry)
            .await
            .map_err(|_| Error::failed("failed delivery persistence failed"))
    }

    async fn load_failed_delivery(&self, id: String) -> Result<Option<DeliveryQueueEntry>> {
        self.delivery
            .load_failed(id)
            .await
            .map_err(|_| Error::failed("failed delivery lookup failed"))
    }

    async fn save_delivery_result(&self, result: DeliveryTerminalResult) -> Result<()> {
        Self::validate_callback_size(&result)?;
        self.delivery
            .save_result(result)
            .await
            .map_err(|_| Error::failed("delivery result persistence failed"))
    }

    async fn load_delivery_result(&self, id: String) -> Result<Option<DeliveryTerminalResult>> {
        self.delivery
            .load_result(id)
            .await
            .map_err(|_| Error::failed("delivery result lookup failed"))
    }

    async fn transcribe(
        &self,
        request: HostTranscriptionRequest,
    ) -> Result<HostTranscriptionResult> {
        if serde_json::to_vec(&request).map_or(true, |bytes| bytes.len() > MAX_TRANSCRIPTION_BYTES)
        {
            return Err(Error::failed("transcription callback exceeds its limit"));
        }
        crate::channels::host::VoiceTranscriber {
            config: Arc::clone(&self.config),
        }
        .transcribe_host(request)
        .await
        .map_err(|_| Error::failed("channel transcription failed"))
    }

    async fn publish_event(&self, publication: HostEventPublication) -> Result<()> {
        Self::validate_callback_size(&publication)?;
        crate::channels::host::OpenHumanEventSink
            .publish_host(&publication.domain, &publication.kind, publication.payload)
            .await
            .map_err(|_| Error::failed("channel event publication failed"))
    }

    async fn dispatch_controller_backend(
        &self,
        request: ChannelBackendRequest,
    ) -> Result<ChannelBackendResult> {
        if serde_json::to_vec(&request).map_or(true, |bytes| bytes.len() > MAX_CONTROLLER_BYTES) {
            return Err(Error::failed("controller callback exceeds its limit"));
        }
        match request {
            ChannelBackendRequest::RemoteControl(request) => {
                let context = self.context()?;
                let tenant = &self.tenant;
                let cancellation = &self.cancellation;
                let dispatch = async {
                    tokio::select! {
                        biased;
                        _ = cancellation.cancelled() => {
                            Err(Error::failed("channel session ended"))
                        }
                        result = crate::channels::host::remote_control::dispatch_callback(
                            context.as_ref(),
                            tenant,
                            cancellation,
                            request,
                        ) => result.map(ChannelBackendResult::RemoteControl),
                    }
                };
                match &self.core_context {
                    Some(core_context) => {
                        crate::core::runtime::CoreContext::scope(Arc::clone(core_context), dispatch)
                            .await
                    }
                    None => dispatch.await,
                }
            }
            ChannelBackendRequest::ProgressiveTransport(_) => Err(Error::failed(
                "progressive transport callback is unavailable",
            )),
            request => {
                crate::channels::controllers::OpenHumanChannelBackend::new((*self.config).clone())
                    .dispatch_callback(request)
                    .await
                    .map_err(|_| Error::failed("channel controller callback failed"))
            }
        }
    }
}
