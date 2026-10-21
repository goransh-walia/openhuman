//! Tests for `inbound.stats`, `inbound.dead` and `inbound.redrive`.

use super::*;
use crate::inbound::fixture::{active_config, Rig, ScriptedTriage, Step};

#[tokio::test]
async fn the_component_switch_refuses_every_operation() {
    let mut config = active_config();
    config.components.eventqueues = false;
    let client = crate::modules::ops::client(config.clone());
    for refused in [
        stats(&config, &client).await.map(|_| ()),
        dead(&config, &client, None).await.map(|_| ()),
        redrive(&config, &client, "x").await.map(|_| ()),
    ] {
        assert!(refused
            .unwrap_err()
            .contains("COMPONENT_DISABLED:eventqueues"));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_operations_reach_the_module() {
    let rig = Rig::new(ScriptedTriage::new(Vec::<Step>::new())).await;
    let config = active_config();

    let depths = stats(&config, &rig.client).await.unwrap();
    assert!(depths.value.queues.iter().any(|q| q.queue == "teq.ingress"));

    let parked = dead(&config, &rig.client, Some(10_000)).await.unwrap();
    assert!(parked.value.items.is_empty());

    let gone = redrive(&config, &rig.client, "teq.dead/none")
        .await
        .unwrap();
    assert!(!gone.value.redriven);
    assert!(redrive(&config, &rig.client, " ")
        .await
        .unwrap_err()
        .contains("id is required"));
    assert!(redrive(&config, &rig.client, "nope")
        .await
        .unwrap_err()
        .starts_with("inbound.redrive:"));
}
