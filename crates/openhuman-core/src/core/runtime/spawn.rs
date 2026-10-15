//! Spawning tasks without dropping who they work for.
//!
//! The ambient [`CoreContext`] is a tokio task-local, so a bare `tokio::spawn`
//! starts its task without it: the child falls back to the process default
//! context. In a single-user process that default is the right user by
//! accident. In a SaaS
//! process it is the operator, so a dropped scope writes one user's work
//! somewhere it does not belong — or, with the SaaS config redirect, fails.
//!
//! [`spawn_scoped`] captures it at the call site and re-enters it in the
//! child. The memory identity is not ambient any more: spawned work that acts
//! on memory is handed its `MemoryIdentity` explicitly.
//! New code in the core spawns through it; `scripts/ci/check-saas-ambient.mjs`
//! ratchets the bare spawns that remain.

use std::future::Future;

use tokio::task::JoinHandle;

use super::CoreContext;

/// Like `tokio::spawn`, but the task keeps the caller's [`CoreContext`].
pub fn spawn_scoped<F>(fut: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(scoped(fut))
}

/// The context a spawned task inherits. In SaaS only the caller's own task
/// scope is carried: an unscoped caller's task gets no context (and so fails
/// closed) rather than the operator's default one.
fn captured() -> Option<std::sync::Arc<CoreContext>> {
    if super::is_saas() {
        CoreContext::scoped()
    } else {
        CoreContext::current()
    }
}

/// `fut`, wrapped to run under the caller's context wherever it is eventually
/// polled. The memory identity is not ambient and so is not carried: work that
/// acts on memory takes it explicitly (the run context, `MemoryIdentity`).
pub fn scoped<F>(fut: F) -> impl Future<Output = F::Output> + Send
where
    F: Future + Send,
    F::Output: Send,
{
    let ctx = captured();
    // Box before the task-local wrapper. Its branching async state otherwise
    // multiplies a large task's inline size before Tokio can box it,
    // overflowing the spawning thread (including the CLI's main stack).
    let fut = Box::pin(fut);
    async move {
        match ctx {
            Some(ctx) => CoreContext::scope(ctx, fut).await,
            None => fut.await,
        }
    }
}

/// What [`CoreContext::propagate`] wraps a future in: the caller's context
/// captured now and re-entered wherever the future is polled.
///
/// Single-user processes carry [`CoreContext::current`]. In SaaS only the
/// caller's own task scope is carried (an unscoped caller's future runs with
/// no context and so fails closed), exactly as [`scoped`] does — but without
/// requiring the future to be `Send`.
pub(crate) fn carry<F: Future>(fut: F) -> impl Future<Output = F::Output> {
    let ctx = captured();
    async move {
        match ctx {
            Some(ctx) => CoreContext::scope(ctx, fut).await,
            None => fut.await,
        }
    }
}

/// Like `tokio::task::spawn_blocking`, but the closure runs under the caller's
/// [`CoreContext`].
pub fn spawn_blocking_scoped<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let ctx = captured();
    let handle = tokio::runtime::Handle::current();
    tokio::task::spawn_blocking(move || match ctx {
        Some(ctx) => handle.block_on(CoreContext::scope(ctx, async move { f() })),
        None => f(),
    })
}

#[cfg(test)]
#[path = "spawn_tests.rs"]
mod tests;
