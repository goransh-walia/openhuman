//! Which `[components]` switch, if any, hides an RPC method.
//!
//! A family a switch removed answers `COMPONENT_DISABLED:<name>` so the UI and
//! CLI can explain it; a family the host merely left out stays
//! indistinguishable from an unregistered method.

use super::{internal_registry, registry_view, visible};

/// `Some(component)` when `method` is registered but hidden because its
/// component is switched off in the ambient scope. Dispatch asks this before
/// reporting an unknown method.
pub fn disabled_component_for_method(method: &str) -> Option<&'static str> {
    let view = registry_view();
    let grouped = view
        .iter()
        .chain(internal_registry().iter())
        .find(|g| g.controller.rpc_method_name() == method)?;
    if visible(grouped) {
        return None;
    }
    let name = crate::core::runtime::ComponentSet::component_for_group(grouped.group)?;
    let ctx = crate::core::runtime::context::CoreContext::current()?;
    if ctx.components().enabled(name) {
        return None;
    }
    log::debug!("[rpc][component-gate] method '{method}' refused, component '{name}' disabled");
    Some(name)
}
