//! Chat threads on the storage backend: the workspace-keyed, synchronous
//! facade the host's callers use, over TinyAgents'
//! [`DriverConversationStore`] (the JSONL file store is gone upstream).
//!
//! Every call resolves the process's storage backend for the workspace
//! ([`crate::storage::local::backend_for`]: the installed backend, else the
//! workspace default) and binds it to the **conversation scope**: the calling
//! tenant's profile scope when there is one, else `local`. Conversations span
//! agents, so the acting agent's scope is deliberately not used: a thread a
//! sub-agent wrote must stay visible to the UI that lists it.
//!
//! The API is the one the file store had (`String` errors, owned arguments,
//! free functions beside [`ConversationStore`]) so the ~50 host callers did not
//! change. Each call blocks on the storage bridge ([`crate::storage::block_on`]);
//! request paths already go through [`super::blocking`], which keeps that off
//! the async workers.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::ConversationPurgeStats;
use tinyagents_session::threads::{
    ConversationError, ConversationPersistence, ConversationResult, ConversationThread,
    CreateConversationThread, CrossThreadHit, DriverConversationStore, ThreadMessage,
    ThreadMessagePatch,
};

use crate::storage::documents::Repo;
use crate::storage::{self, Scope, StorageError};
use tinystoragedrivers::ErrorKind;

const DOMAIN: &str = "threads::store";
const LOG_PREFIX: &str = "[conversations]";

/// The storage scope conversations live under: the calling tenant's profile,
/// else `local`; refused in SaaS mode with no profile.
fn conversation_scope() -> Result<Scope, String> {
    let tenant = crate::core::runtime::current_tenant().map_err(|e| e.to_string())?;
    storage::scope_from(
        tenant.profile.as_deref(),
        None,
        crate::core::runtime::mode::is_saas(),
    )
    .map_err(|e| e.to_string())
}

/// What an operation addresses, to word a not-found error as the file store did.
#[derive(Clone, Copy)]
struct Subject<'a> {
    thread: Option<&'a str>,
    message: Option<&'a str>,
}

const NONE: Subject<'static> = Subject {
    thread: None,
    message: None,
};

const fn thread(id: &str) -> Subject<'_> {
    Subject {
        thread: Some(id),
        message: None,
    }
}

const fn message<'a>(thread: &'a str, message: &'a str) -> Subject<'a> {
    Subject {
        thread: Some(thread),
        message: Some(message),
    }
}

/// The error text callers match on: a missing thread or message reads
/// `thread <id> not found` / `message <id> not found in thread <id>` (the
/// wording the file store used and `threads::ops` maps to a typed error);
/// anything else is the driver's own message.
fn describe(error: &ConversationError, subject: Subject<'_>) -> String {
    if let ConversationError::Driver(inner) = error {
        if inner.kind() == ErrorKind::NotFound {
            let text = inner.to_string();
            match (subject.thread, subject.message) {
                (Some(t), Some(m)) if text.contains("message") => {
                    return format!("message {m} not found in thread {t}")
                }
                (Some(t), _) => return format!("thread {t} not found"),
                _ => {}
            }
        }
    }
    error.to_string()
}

/// A conversation store bound to one workspace's storage backend.
#[derive(Clone)]
pub struct ConversationStore {
    workspace_dir: PathBuf,
}

impl ConversationStore {
    /// A store for `workspace_dir`. Nothing is opened until the first call.
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }

    /// The driver store for this call, its schema declared once per backend
    /// and scope, and the legacy JSONL conversations imported once.
    fn open(&self) -> Result<DriverConversationStore, String> {
        let scope = conversation_scope()?;
        let backend = storage::local::backend_for(&self.workspace_dir)
            .map_err(|e| format!("{LOG_PREFIX} open the storage backend: {e:#}"))?;
        let scoped = backend
            .for_scope(&scope)
            .map_err(|e| format!("{LOG_PREFIX} open the storage scope: {e}"))?;
        let repo = Repo::on(&backend, &scope, DOMAIN, super::legacy_import::collections)
            .map_err(|e| format!("{LOG_PREFIX} {e:#}"))?;
        let store = DriverConversationStore::new(scoped.clone());
        repo.once_per_backend("schema", || {
            let docs = Arc::clone(scoped.documents());
            storage::block_on(async move {
                DriverConversationStore::prepare_schema(docs.as_ref()).await?;
                DriverConversationStore::prepare_append_schema(docs.as_ref()).await
            })
            .map_err(|e| anyhow::anyhow!("declare the conversation schema: {e}"))
        })
        .map_err(|e| format!("{LOG_PREFIX} {e:#}"))?;
        super::legacy_import::import_once(&self.workspace_dir, &repo, &store)
            .map_err(|e| format!("{LOG_PREFIX} legacy import: {e:#}"))?;
        Ok(store)
    }

    fn run<T, F, Fut>(
        &self,
        operation: &'static str,
        subject: Subject<'_>,
        f: F,
    ) -> Result<T, String>
    where
        F: FnOnce(DriverConversationStore) -> Fut,
        Fut: std::future::Future<Output = ConversationResult<T>> + Send + 'static,
        T: Send + 'static,
    {
        let store = self.open()?;
        let future = f(store);
        let outcome = storage::block_on(async move { Ok(future.await) })
            .map_err(|e: StorageError| format!("{LOG_PREFIX} {operation}: {e}"))?;
        outcome.map_err(|e| {
            log::debug!("{LOG_PREFIX} {operation} failed: {e}");
            describe(&e, subject)
        })
    }

    /// Creates the thread or returns the existing one.
    pub fn ensure_thread(
        &self,
        request: CreateConversationThread,
    ) -> Result<ConversationThread, String> {
        self.run("ensure_thread", NONE, |s| async move {
            s.ensure_thread(request).await
        })
    }

    /// Every live thread, newest first.
    pub fn list_threads(&self) -> Result<Vec<ConversationThread>, String> {
        self.run(
            "list_threads",
            NONE,
            |s| async move { s.list_threads().await },
        )
    }

    /// A thread's messages in append order; empty for an unknown thread.
    pub fn get_messages(&self, thread_id: &str) -> Result<Vec<ThreadMessage>, String> {
        let id = thread_id.to_string();
        self.run("get_messages", thread(thread_id), |s| async move {
            if s.get_thread(&id).await?.is_none() {
                return Ok(Vec::new());
            }
            s.get_messages(&id).await
        })
    }

    /// Cross-thread substring search over message content.
    pub fn search_cross_thread_messages(
        &self,
        query: &str,
        limit: usize,
        exclude_thread_id: Option<&str>,
    ) -> Result<Vec<CrossThreadHit>, String> {
        let query = query.to_string();
        let exclude = exclude_thread_id.map(str::to_string);
        self.run("search_cross_thread_messages", NONE, |s| async move {
            s.search_cross_thread_messages(&query, limit, exclude.as_deref())
                .await
        })
    }

    /// Appends a message to a thread.
    pub fn append_message(
        &self,
        thread_id: &str,
        message: ThreadMessage,
    ) -> Result<ThreadMessage, String> {
        let id = thread_id.to_string();
        self.run("append_message", thread(thread_id), |s| async move {
            s.append_message(&id, message).await
        })
    }

    /// Renames a thread.
    pub fn update_thread_title(
        &self,
        thread_id: &str,
        title: &str,
        updated_at: &str,
    ) -> Result<ConversationThread, String> {
        let (id, title, at) = (
            thread_id.to_string(),
            title.to_string(),
            updated_at.to_string(),
        );
        self.run("update_thread_title", thread(thread_id), |s| async move {
            s.update_thread_title(&id, &title, &at).await
        })
    }

    /// Sets a thread's working directory.
    pub fn update_thread_working_dir(
        &self,
        thread_id: &str,
        working_dir: Option<String>,
        updated_at: &str,
    ) -> Result<ConversationThread, String> {
        let (id, at) = (thread_id.to_string(), updated_at.to_string());
        self.run(
            "update_thread_working_dir",
            thread(thread_id),
            |s| async move { s.update_thread_working_dir(&id, working_dir, &at).await },
        )
    }

    /// Replaces a thread's labels.
    pub fn update_thread_labels(
        &self,
        thread_id: &str,
        labels: Vec<String>,
        updated_at: &str,
    ) -> Result<ConversationThread, String> {
        let (id, at) = (thread_id.to_string(), updated_at.to_string());
        self.run("update_thread_labels", thread(thread_id), |s| async move {
            s.update_thread_labels(&id, labels, &at).await
        })
    }

    /// Patches one message.
    pub fn update_message(
        &self,
        thread_id: &str,
        message_id: &str,
        patch: ThreadMessagePatch,
    ) -> Result<ThreadMessage, String> {
        let (id, mid) = (thread_id.to_string(), message_id.to_string());
        self.run(
            "update_message",
            message(thread_id, message_id),
            |s| async move { s.update_message(&id, &mid, patch).await },
        )
    }

    /// Removes a message and everything after it; `None` for an unknown id.
    pub fn delete_messages_from(
        &self,
        thread_id: &str,
        message_id: &str,
    ) -> Result<Option<usize>, String> {
        let (id, mid) = (thread_id.to_string(), message_id.to_string());
        self.run(
            "delete_messages_from",
            message(thread_id, message_id),
            |s| async move { s.delete_messages_from(&id, &mid).await },
        )
    }

    /// Soft-deletes a thread; whether it existed.
    pub fn delete_thread(&self, thread_id: &str, deleted_at: &str) -> Result<bool, String> {
        let (id, at) = (thread_id.to_string(), deleted_at.to_string());
        self.run("delete_thread", thread(thread_id), |s| async move {
            s.delete_thread(&id, &at).await
        })
    }

    /// Removes every thread.
    pub fn purge_threads(&self) -> Result<ConversationPurgeStats, String> {
        self.run("purge_threads", NONE, |s| async move {
            s.purge_threads().await.map(ConversationPurgeStats::from)
        })
    }
}

fn store(workspace_dir: PathBuf) -> ConversationStore {
    ConversationStore::new(workspace_dir)
}

/// Free-function shim around [`ConversationStore::ensure_thread`].
pub fn ensure_thread(
    workspace_dir: PathBuf,
    request: CreateConversationThread,
) -> Result<ConversationThread, String> {
    store(workspace_dir).ensure_thread(request)
}

/// Free-function shim around [`ConversationStore::list_threads`].
pub fn list_threads(workspace_dir: PathBuf) -> Result<Vec<ConversationThread>, String> {
    store(workspace_dir).list_threads()
}

/// Free-function shim around [`ConversationStore::get_messages`].
pub fn get_messages(workspace_dir: PathBuf, thread_id: &str) -> Result<Vec<ThreadMessage>, String> {
    store(workspace_dir).get_messages(thread_id)
}

/// Free-function shim around [`ConversationStore::append_message`].
pub fn append_message(
    workspace_dir: PathBuf,
    thread_id: &str,
    message: ThreadMessage,
) -> Result<ThreadMessage, String> {
    store(workspace_dir).append_message(thread_id, message)
}

/// Free-function shim around [`ConversationStore::update_thread_title`].
pub fn update_thread_title(
    workspace_dir: PathBuf,
    thread_id: &str,
    title: &str,
    updated_at: &str,
) -> Result<ConversationThread, String> {
    store(workspace_dir).update_thread_title(thread_id, title, updated_at)
}

/// Free-function shim around [`ConversationStore::update_thread_labels`].
pub fn update_thread_labels(
    workspace_dir: PathBuf,
    thread_id: &str,
    labels: Vec<String>,
    updated_at: &str,
) -> Result<ConversationThread, String> {
    store(workspace_dir).update_thread_labels(thread_id, labels, updated_at)
}

/// Free-function shim around [`ConversationStore::update_thread_working_dir`].
pub fn update_thread_working_dir(
    workspace_dir: PathBuf,
    thread_id: &str,
    working_dir: Option<String>,
    updated_at: &str,
) -> Result<ConversationThread, String> {
    store(workspace_dir).update_thread_working_dir(thread_id, working_dir, updated_at)
}

/// Free-function shim around [`ConversationStore::update_message`].
pub fn update_message(
    workspace_dir: PathBuf,
    thread_id: &str,
    message_id: &str,
    patch: ThreadMessagePatch,
) -> Result<ThreadMessage, String> {
    store(workspace_dir).update_message(thread_id, message_id, patch)
}

/// Free-function shim around [`ConversationStore::delete_messages_from`].
pub fn delete_messages_from(
    workspace_dir: PathBuf,
    thread_id: &str,
    message_id: &str,
) -> Result<Option<usize>, String> {
    store(workspace_dir).delete_messages_from(thread_id, message_id)
}

/// Free-function shim around [`ConversationStore::purge_threads`].
pub fn purge_threads(workspace_dir: PathBuf) -> Result<ConversationPurgeStats, String> {
    store(workspace_dir).purge_threads()
}

/// Free-function shim around [`ConversationStore::delete_thread`].
pub fn delete_thread(
    workspace_dir: PathBuf,
    thread_id: &str,
    deleted_at: &str,
) -> Result<bool, String> {
    store(workspace_dir).delete_thread(thread_id, deleted_at)
}

/// Whether `path` is the legacy conversation root of `workspace_dir`.
pub(super) fn legacy_root(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join("memory").join("conversations")
}
