use super::channel_has_approval_surface;
use tinychannels_bus::ChannelCapabilities;

#[test]
fn configured_instances_with_chat_approvals_have_an_approval_surface() {
    let mut capabilities = ChannelCapabilities::unknown();
    capabilities.chat_approvals = true;
    assert!(channel_has_approval_surface(&capabilities));
}

#[test]
fn unknown_or_unconfigured_instances_do_not_have_an_approval_surface() {
    assert!(!channel_has_approval_surface(
        &ChannelCapabilities::unknown()
    ));
}
