//! OpenHuman's host-facing channel operations.
//!
//! Provider implementations live in TinyChannels. This small trait keeps the
//! host's delivery consumers independent from the provider runtime and lets a
//! scoped module proxy satisfy cron, proactive delivery, and event subscribers.

use async_trait::async_trait;
pub use tinychannels_bus::{ChannelMessage, SendMessage};

/// Host-side operations used by OpenHuman's channel delivery consumers.
///
/// `listen` remains available to focused test doubles and legacy local
/// adapters. Production module-backed channels receive inbound traffic through
/// their identity-scoped callback object instead.
#[async_trait]
pub trait Channel: Send + Sync {
    /// Stable provider name (for example, `telegram`).
    fn name(&self) -> &str;

    /// Configured default target for recipient-less proactive sends.
    fn proactive_target(&self) -> Option<String> {
        None
    }

    /// Deliver one message through the configured provider.
    async fn send(&self, message: &SendMessage) -> anyhow::Result<()>;

    /// Legacy listener seam used by isolated test doubles.
    async fn listen(&self, tx: tokio::sync::mpsc::Sender<ChannelMessage>) -> anyhow::Result<()>;

    /// Whether the configured provider is currently healthy.
    async fn health_check(&self) -> bool {
        true
    }

    /// Start a provider typing indicator.
    async fn start_typing(&self, _recipient: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// Stop a provider typing indicator.
    async fn stop_typing(&self, _recipient: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// Start a module-owned periodic refresh and return its opaque lifecycle
    /// handle. Implementations without provider-side refresh support return
    /// `None`; the host does not create a local refresh timer.
    async fn begin_typing_refresh(
        &self,
        _recipient: &str,
        _interval_secs: u64,
    ) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    /// Stop one refresh lifecycle by its opaque handle. Handles let concurrent
    /// turns type in the same conversation without canceling each other's
    /// provider task.
    async fn end_typing_refresh(&self, _handle: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// Instance-specific product interaction capabilities.
    fn capabilities(&self) -> tinychannels_bus::ChannelCapabilities {
        tinychannels_bus::capabilities_for(self.name())
    }

    /// Whether native emoji reactions are supported.
    fn supports_reactions(&self) -> bool {
        false
    }

    /// Whether progressive draft updates are supported.
    fn supports_draft_updates(&self) -> bool {
        false
    }

    /// Send the initial progressive draft and return its provider message id.
    async fn send_draft(&self, _message: &SendMessage) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    /// Update a progressive draft.
    async fn update_draft(
        &self,
        _recipient: &str,
        _message_id: &str,
        _text: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// Finalize a progressive draft.
    async fn finalize_draft(
        &self,
        _recipient: &str,
        _message_id: &str,
        _text: &str,
        _thread_ts: Option<&str>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// Convert and send one message using the owning provider's idempotency
    /// contract. Module-backed implementations override this to perform
    /// `PrepareOutboundIntent` and `SendChannelIntent` through TinyBus.
    async fn send_idempotent(&self, message: &SendMessage) -> anyhow::Result<()> {
        self.send(message).await
    }
}

/// Extension helpers for host delivery consumers.
#[async_trait]
pub trait ChannelSendExt: Channel {
    /// Send one message with the provider's deterministic idempotency handling.
    async fn send_with_outbound_intent(&self, message: &SendMessage) -> anyhow::Result<()> {
        Channel::send_idempotent(self, message).await
    }
}

impl<T: Channel + ?Sized> ChannelSendExt for T {}

#[cfg(test)]
#[path = "traits_tests.rs"]
mod tests;
