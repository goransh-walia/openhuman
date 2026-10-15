//! The explicit context a controller handler runs under.
//!
//! Every registered controller handler is `fn(DispatchCtx, Map) -> ControllerFuture`.
//! The transport that accepted the call (the JSON-RPC server, the CLI, the
//! device tunnel, [`CoreRuntime::invoke_in`](super::CoreRuntime::invoke_in))
//! builds the [`DispatchCtx`] and hands it to
//! [`invoke_method`](crate::core::invoke::invoke_method); the registry passes it
//! on to the handler. Nothing in between reads it from task state, so two
//! concurrent dispatches for different tenants cannot see each other's value.
//!
//! A handler hands the context to whatever it calls (`ctx.core()` for the
//! [`CoreContext`], or the helpers built on it) rather than reaching for an
//! ambient one.

use std::sync::Arc;

use super::CoreContext;

/// The per-call context a controller handler receives.
///
/// Cheap to clone (one `Arc`). It is `unbound` only when no [`CoreContext`]
/// exists at all: a unit test that never booted the core, or a call made
/// before boot. A booted host always dispatches with a bound context.
#[derive(Clone, Default)]
pub struct DispatchCtx {
    core: Option<Arc<CoreContext>>,
}

impl DispatchCtx {
    /// Dispatch under `core`: the tenant's context for a SaaS request, an
    /// agent's derived context for an embedded call, the host's own otherwise.
    pub fn new(core: Arc<CoreContext>) -> Self {
        Self { core: Some(core) }
    }

    /// Dispatch with no context. Only for tests and pre-boot callers; every
    /// reader of the context treats this as "nothing configured here".
    pub fn unbound() -> Self {
        Self { core: None }
    }

    /// Dispatch under the process default context (the first one built), or
    /// unbound before any exists. What the CLI and the desktop shell use: a
    /// single-user host has exactly one tenant.
    pub fn process_default() -> Self {
        Self {
            core: CoreContext::default_context(),
        }
    }

    /// The ambient context of the calling task, or unbound. In SaaS only the
    /// task's own scope counts.
    ///
    /// Transitional (#7801): for a caller that has no context of its own to
    /// hand over yet. Every use is a reader still to be moved onto an explicit
    /// carrier, and the function goes away with the ambient context.
    pub fn ambient() -> Self {
        // SaaS reads only the task's own scope, never the process default, so
        // work that lost its tenant fails closed instead of acting as the operator.
        let core = if super::is_saas() {
            CoreContext::scoped()
        } else {
            CoreContext::current()
        };
        Self { core }
    }

    /// The context readers act under: the bound one, else (outside SaaS) the
    /// process default, else none.
    ///
    /// An unbound context outside SaaS stands for the host's one context, as an
    /// unscoped task always did, so tests and pre-boot callers that never bound
    /// one read the default. In SaaS an unbound context stays `None`: work that
    /// lost its tenant must fail closed rather than act as the operator.
    pub fn resolved(&self) -> Option<Arc<CoreContext>> {
        match &self.core {
            Some(core) => Some(Arc::clone(core)),
            None if super::is_saas() => None,
            None => CoreContext::default_context(),
        }
    }

    /// The configuration an embedder supplied for the context this call runs
    /// under, if any. See [`CoreContext::embedder_config`].
    pub fn embedder_config(&self) -> Option<crate::config::Config> {
        self.resolved()
            .and_then(|core| core.embedder_config().cloned())
    }

    /// Loads the config this call runs against: the bound context's embedder
    /// config when it has one, else the persisted config. Fails closed in
    /// SaaS when no context is bound. See
    /// [`load_config_in`](crate::config::ops::load_config_in).
    pub async fn load_config(&self) -> Result<crate::config::Config, String> {
        crate::config::ops::load_config_in(self).await
    }

    /// A non-owning handle on the bound context, for state that outlives the
    /// call (a session, a built turn). The tenant lease is the strong count of
    /// the context ([`CoreContext::tenant_in_use`]), so a long-lived holder
    /// must not keep it alive.
    pub fn downgrade(&self) -> Option<std::sync::Weak<CoreContext>> {
        self.core.as_ref().map(Arc::downgrade)
    }

    /// Dispatch under the context `weak` still points at, else the calling
    /// task's ambient one (a holder that was never bound, or whose tenant is
    /// gone).
    pub fn from_weak(weak: Option<&std::sync::Weak<CoreContext>>) -> Self {
        match weak.and_then(std::sync::Weak::upgrade) {
            Some(core) => Self::new(core),
            None => Self::ambient(),
        }
    }

    /// The context this call runs under, if any.
    pub fn core(&self) -> Option<&Arc<CoreContext>> {
        self.core.as_ref()
    }

    /// Takes the context out, for a caller that spawns a task and must hand
    /// it over explicitly.
    pub fn into_core(self) -> Option<Arc<CoreContext>> {
        self.core
    }
}

impl From<Arc<CoreContext>> for DispatchCtx {
    fn from(core: Arc<CoreContext>) -> Self {
        Self::new(core)
    }
}

impl std::fmt::Debug for DispatchCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DispatchCtx")
            .field("bound", &self.core.is_some())
            .finish()
    }
}

#[cfg(test)]
#[path = "dispatch_ctx_tests.rs"]
mod tests;
