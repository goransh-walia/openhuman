use super::*;
use std::sync::Arc;
use tokio::sync::{oneshot, Notify};

struct DeferredTyping {
    started: Notify,
    release: Notify,
    stopped: std::sync::Mutex<Option<oneshot::Sender<String>>>,
}
#[async_trait::async_trait]
impl crate::channels::Channel for DeferredTyping {
    fn name(&self) -> &str {
        "fixture"
    }
    async fn send(&self, _: &crate::channels::SendMessage) -> anyhow::Result<()> {
        Ok(())
    }
    async fn listen(
        &self,
        _: tokio::sync::mpsc::Sender<traits::ChannelMessage>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    async fn begin_typing_refresh(&self, _: &str, _: u64) -> anyhow::Result<Option<String>> {
        self.started.notify_one();
        self.release.notified().await;
        Ok(Some("module-handle".into()))
    }
    async fn end_typing_refresh(&self, handle: &str) -> anyhow::Result<()> {
        if let Some(sender) = self.stopped.lock().unwrap().take() {
            let _ = sender.send(handle.into());
        }
        Ok(())
    }
}

#[tokio::test]
async fn canceled_waiter_closes_handle_returned_by_in_flight_typing_start() {
    let (sender, receiver) = oneshot::channel();
    let channel = Arc::new(DeferredTyping {
        started: Notify::new(),
        release: Notify::new(),
        stopped: std::sync::Mutex::new(Some(sender)),
    });
    let cancellation = tokio_util::sync::CancellationToken::new();
    let waiter =
        spawn_scoped_typing_task(channel.clone(), "recipient".into(), cancellation.clone());
    channel.started.notified().await;
    waiter.abort();
    cancellation.cancel();
    channel.release.notify_one();
    assert_eq!(
        tokio::time::timeout(std::time::Duration::from_secs(2), receiver)
            .await
            .unwrap()
            .unwrap(),
        "module-handle"
    );
}
