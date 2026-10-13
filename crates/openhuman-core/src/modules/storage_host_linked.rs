//! The storage host as a *linked* tinybus module.
//!
//! `ai.tinyhumans.tinystorage.Storage` has a confidential object (`Secrets`),
//! and the broker delivers a confidential call only to an attested recipient:
//! a loaded module whose artifact the host verified, or code linked into the
//! host executable. The host's own client connection is neither, so a loaded
//! module (tinymcp keeps its server environment and OAuth bundles there) was
//! refused `NotAttested` and could not persist anything that mattered.
//!
//! This file wraps the host's existing [`StorageHost`] in the smallest possible
//! linked module. `ModuleHost::attach_linked_with_config` admits it with the
//! executable's own digest, which makes it an attested recipient, and its setup
//! serves the same four objects. Every decision (caller, scope, grant, fence,
//! audit) stays in [`StorageHost`]; the module adds none and takes nothing from
//! a request.
//!
//! The module runs on its own runtime, so each call is handed back to the
//! host's runtime (`ObjectService::runtime`), where the backend was opened.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tinybus::module::ModuleHost;
use tinystoragedrivers_bus::STORAGE_INTERFACE;

use super::storage_host::{serve_objects, StorageHost};

/// What the module's setup serves. Set by [`attach`] just before the module is
/// admitted, because a linked module's setup is a plain function with no state.
static PENDING: Mutex<Option<(Arc<StorageHost>, tokio::runtime::Handle)>> = Mutex::new(None);

/// How long [`attach`] waits for the module to claim the interface name.
const READY_DEADLINE: Duration = Duration::from_secs(5);

async fn setup(connection: tinybus::Connection) -> tinybus::Result<()> {
    let (host, runtime) = PENDING
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .clone()
        .ok_or_else(|| tinybus::Error::failed("storage host has nothing to serve"))?;
    serve_objects(&connection, &host, Some(runtime)).await
}

mod export {
    tinybus_module::module_export_static! {
        setup = super::setup,
        worker_threads = 1,
        provides = ["ai.tinyhumans.tinystorage.Storage"],
        methods = [],
        signals = [],
        requires = [],
        optional = [],
        lazy = false,
    }
}

/// Admits the storage host as a linked module and waits until it serves.
///
/// # Errors
///
/// When the module is refused, or does not claim the interface name in time.
pub(super) async fn attach(
    modules: &ModuleHost,
    connection: &tinybus::Connection,
    host: Arc<StorageHost>,
) -> tinybus::Result<()> {
    *PENDING.lock().unwrap_or_else(|poison| poison.into_inner()) =
        Some((host, tokio::runtime::Handle::current()));
    modules.attach_linked_with_config(export::linked_module()?, serde_json::json!({}))?;
    let name = STORAGE_INTERFACE;
    let deadline = tokio::time::Instant::now() + READY_DEADLINE;
    loop {
        if matches!(connection.name_owner(&name).await, Ok(Some(_))) {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(tinybus::Error::failed(
                "linked storage module did not claim its name",
            ));
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[cfg(test)]
#[path = "storage_host_linked_tests.rs"]
mod tests;
