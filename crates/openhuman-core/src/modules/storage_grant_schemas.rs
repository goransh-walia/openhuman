//! The storage grants of the loadable modules, generated from their schemas.
//!
//! A module reaches storage only through `ai.tinyhumans.tinystorage.Storage`
//! (`storage_host`) and its authority is a [`StorageGrant`]. Each module owner
//! declares what it stores in `storage_schema()` (collections with their
//! indexes, streams, blob and secret prefixes, each with a scope class); the
//! host composes those into the registry ([`crate::storage::schema`]) and this
//! file turns the **module-scoped** items of the module's owner into the grant.
//! The hand-written per-module tables this replaces could drift from what the
//! module actually declares; a generated grant cannot.
//!
//! Secrets are the one thing a schema cannot name for tinymcp (a secret cannot
//! be renamed by a migration step and sits outside the owner prefix), so the
//! row lists those beside the schema.
//!
//! The `mod <module>` blocks hold the registry id and the v1 names the host's
//! one-time legacy-file importers (`storage_import_*`) still write: the
//! importers write the module's v1 shapes and the owner's steps carry them
//! forward (`ensure_migrated` in `storage_host`).

use std::sync::Arc;

use tinystoragedrivers::queue::QueueManifest;

use super::storage_grants::StorageGrant;
use crate::storage::schema::module_items;

/// One module's row: its registry id, the schema owner that declares its
/// storage, secret prefixes the schema cannot name, and the queue manifest the
/// host declares for it (`None`: the module gets no `Queues` object).
struct Row {
    module: &'static str,
    owner: &'static str,
    secrets: &'static [&'static str],
    queues: QueueManifestSource,
}

/// Where a row's queue grant comes from, read when the grant is built (at
/// bind), so a manifest the host loads from configuration is current.
pub(super) type QueueManifestSource = Option<fn() -> Option<QueueGrant>>;

/// The queues a module is granted: the manifest the host declares on its
/// scope, and where they live.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct QueueGrant {
    /// The queues the module may use; the storage server refuses any other.
    pub manifest: Arc<QueueManifest>,
    /// `Storage` keeps them in the scope's documents; a broker URL moves them.
    pub backend: tinystoragedrivers::QueueConfig,
}

const ROWS: &[Row] = &[
    Row {
        module: tinychannels::MODULE_ID,
        owner: "channels",
        secrets: &[],
        queues: None,
    },
    Row {
        module: tinymcp::MODULE_ID,
        owner: "mcp",
        secrets: &["mcp_client_env/", "mcp_cache_fingerprint_key"],
        queues: None,
    },
    Row {
        module: tinyconnectors::MODULE_ID,
        owner: "connectors",
        secrets: &[],
        queues: None,
    },
    Row {
        module: tinywallet::MODULE_ID,
        owner: "wallet",
        secrets: &[],
        queues: None,
    },
    Row {
        module: tinybox::MODULE_ID,
        owner: "box",
        secrets: &[],
        queues: None,
    },
    Row {
        module: tinyjuice::MODULE_ID,
        owner: "juice",
        secrets: &[],
        queues: None,
    },
    // The inbound event queues keep their pipeline state in `teq.*`
    // collections and their work in the queues the configured manifest
    // declares (the shipped `queues.json` unless `[inbound] queues_path`).
    #[cfg(feature = "inbound-queues")]
    Row {
        module: tinyeventqueues::MODULE_ID,
        owner: tinyeventqueues_bus::storage::OWNER,
        secrets: &[],
        queues: Some(crate::inbound::queue_grant),
    },
];

/// The schema owner whose declarations hold `module_id`'s storage.
pub(super) fn owner_of(module_id: &str) -> Option<&'static str> {
    ROWS.iter()
        .find(|row| row.module == module_id)
        .map(|row| row.owner)
}

/// Carries what a legacy-file importer just wrote into `module_id`'s scope
/// (v1 names and shapes) forward to the owner's v2 schema.
pub(super) async fn migrate_imported(ctx: &crate::core::runtime::DispatchCtx, module_id: &str) {
    let (Some(owner), Some(backend)) = (owner_of(module_id), crate::storage::installed()) else {
        return;
    };
    let scope = match super::storage_host::current_module_scope(ctx, module_id) {
        Ok(scope) => scope,
        Err(error) => {
            tracing::warn!(module = module_id, kind = ?error.kind(), "[storage_grant] no module scope to migrate");
            return;
        }
    };
    if let Err(error) = crate::storage::boot::remigrate_owner(&backend, &scope, owner).await {
        tracing::warn!(
            module = module_id,
            %error,
            "[storage_grant] could not migrate the imported legacy records; retried at the next import"
        );
    }
}

/// The grant generated for `module_id`, or `None` for a module with no row.
pub(super) fn grant_for(module_id: &str) -> Option<StorageGrant> {
    let row = ROWS.iter().find(|row| row.module == module_id)?;
    let queues = row.queues.and_then(|source| source());
    let items = module_items(row.owner);
    let mut secret_prefixes: Vec<String> = row.secrets.iter().map(|s| (*s).to_owned()).collect();
    secret_prefixes.extend(items.secrets);
    Some(StorageGrant {
        collections: items.collections,
        stream_prefixes: items.streams,
        blob_prefixes: items.blobs,
        secret_prefixes,
        writable: true,
        queues: queues.as_ref().map(|grant| Arc::clone(&grant.manifest)),
        queue_backend: queues.map(|grant| grant.backend),
    })
}

pub(super) mod tinychannels {
    pub(crate) const MODULE_ID: &str = "tinychannels";
}

pub(super) mod tinymcp {
    pub(crate) const MODULE_ID: &str = "tinymcp";
}

pub(super) mod tinyconnectors {
    pub(crate) const MODULE_ID: &str = "tinyconnectors";
    /// The v1 preference collection the importer writes.
    pub(crate) const PREFS_COLLECTION: &str = "composio-user-scopes";
    /// The v1 trigger-archive stream prefix the importer writes.
    pub(crate) const TRIGGER_STREAM_PREFIX: &str = "triggers/";
}

pub(super) mod tinywallet {
    pub(crate) const MODULE_ID: &str = "tinywallet";
    /// The v1 payments stream the importer writes.
    pub(crate) const PAYMENTS_STREAM: &str = "x402.payments";
}

pub(super) mod tinybox {
    pub(crate) const MODULE_ID: &str = "tinybox";
    /// The v1 jail-index collection the importer writes.
    pub(crate) const JAIL_INDEX_COLLECTION: &str = "tinybox_jail_index";
}

#[cfg(feature = "inbound-queues")]
pub(super) mod tinyeventqueues {
    pub(crate) const MODULE_ID: &str = "tinyeventqueues";
}

pub(super) mod tinyjuice {
    pub(crate) const MODULE_ID: &str = "tinyjuice";
    /// The v1 CCR blob prefix the importer writes.
    pub(crate) const CCR_BLOB_PREFIX: &str = "tinyjuice_ccr/";
}

#[cfg(test)]
#[path = "storage_grant_schemas_tests.rs"]
mod tests;
