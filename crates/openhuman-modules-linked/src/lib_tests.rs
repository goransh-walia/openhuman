use super::*;

#[test]
fn every_enabled_feature_registers_exactly_its_own_id() {
    let ids: Vec<&str> = linked_modules().iter().map(|(id, _)| *id).collect();
    let expected: Vec<&str> = [cfg!(feature = "tinydocs").then_some("tinydocs")]
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(ids, expected);
}

#[test]
fn install_is_idempotent() {
    install();
    install();
}

#[cfg(feature = "tinydocs")]
#[test]
fn install_makes_the_factory_visible_to_the_core() {
    install();
    assert!(openhuman_rpc::embed::modules::is_linked_registered(
        "tinydocs"
    ));
}
