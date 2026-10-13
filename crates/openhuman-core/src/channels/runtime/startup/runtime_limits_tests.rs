use super::*;

#[tokio::test]
#[ignore = "blocked on the linked tinychannels-module fixture (tinybus epic, #7739)"]
async fn runtime_limits_come_from_the_linked_channel_module() {
    let client = crate::channels::module_fixture::client().await;

    let limits = prepare_runtime_limits(&client, 1, 100)
        .await
        .expect("the linked module should prepare valid runtime limits");

    assert_eq!(limits, (30, 64));
}
