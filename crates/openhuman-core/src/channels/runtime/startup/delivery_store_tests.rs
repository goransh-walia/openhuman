use super::*;
use tinychannels_bus::{ChannelOutboundIntent, DeliveryDurability, OutboundPayload};

fn entry(id: &str, at: u64) -> DeliveryQueueEntry {
    DeliveryQueueEntry::new(
        id,
        ChannelOutboundIntent {
            idempotency_key: id.to_owned(),
            channel_id: "telegram".into(),
            conversation_id: "chat-1".into(),
            reply_to_id: None,
            thread_id: None,
            durability: DeliveryDurability::Required,
            payload: OutboundPayload::Text {
                text: "hello".into(),
            },
        },
        at,
    )
}

fn store(path: &Path) -> ChannelDeliveryStore {
    ChannelDeliveryStore {
        path: path.to_path_buf(),
        gate: Arc::new(Mutex::new(())),
    }
}

#[tokio::test]
async fn pending_delivery_is_durable_ordered_and_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("delivery.json");
    let first = store(&path);
    first.save_pending(entry("later", 20)).await.unwrap();
    first.save_pending(entry("earlier", 10)).await.unwrap();
    first.save_pending(entry("earlier", 10)).await.unwrap();

    let reopened = store(&path);
    assert_eq!(
        reopened.load_pending("earlier".into()).await.unwrap(),
        Some(entry("earlier", 10))
    );
    assert_eq!(
        reopened
            .load_pending_all()
            .await
            .unwrap()
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["earlier", "later"]
    );
}

#[tokio::test]
async fn changed_intent_cannot_reuse_a_pending_delivery_id() {
    let dir = tempfile::tempdir().unwrap();
    let store = store(&dir.path().join("delivery.json"));
    store.save_pending(entry("same", 1)).await.unwrap();
    let mut changed = entry("same", 1);
    changed.intent.conversation_id = "other-chat".into();
    assert!(store.save_pending(changed).await.is_err());
    assert_eq!(
        store.load_pending_all().await.unwrap(),
        vec![entry("same", 1)]
    );
}

#[tokio::test]
async fn terminal_result_is_replayed_and_conflicts_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let store = store(&dir.path().join("delivery.json"));
    let result = DeliveryTerminalResult {
        id: "done".into(),
        intent_fingerprint: "abc123".into(),
        completed_at_unix_ms: now_unix_ms(),
    };
    store.save_result(result.clone()).await.unwrap();
    store.save_result(result.clone()).await.unwrap();
    assert_eq!(
        store.load_result("done".into()).await.unwrap(),
        Some(result.clone())
    );

    let conflict = DeliveryTerminalResult {
        intent_fingerprint: "different".into(),
        ..result
    };
    assert!(store.save_result(conflict).await.is_err());
}

#[tokio::test]
async fn removal_and_failed_delivery_are_idempotent_and_durable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("delivery.json");
    let delivery_store = store(&path);
    let failed = entry("failed", 1);
    delivery_store.save_failed(failed.clone()).await.unwrap();
    delivery_store.save_failed(failed.clone()).await.unwrap();
    delivery_store
        .save_pending(entry("pending", 2))
        .await
        .unwrap();
    delivery_store
        .remove_pending("pending".into())
        .await
        .unwrap();
    delivery_store
        .remove_pending("pending".into())
        .await
        .unwrap();

    let reopened = store(&path);
    assert_eq!(
        reopened.load_failed("failed".into()).await.unwrap(),
        Some(failed)
    );
    assert!(reopened
        .load_pending("pending".into())
        .await
        .unwrap()
        .is_none());
}
