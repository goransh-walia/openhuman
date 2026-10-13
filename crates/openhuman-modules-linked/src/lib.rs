//! Linked modules for local development and local tests.
//!
//! Module implementation crates never become core dependencies. This crate is
//! the one place they are linked, and only into a host binary built for local
//! work: `openhuman-cli` with its non-product `linked-modules` feature. A
//! shipped or CI build loads pinned, digest-checked releases instead and never
//! reaches this crate.
//!
//! [`install`] registers one factory per enabled module with the core's
//! `register_linked` seam. Registration alone changes nothing: a module is
//! served from the executable only when the configuration asks for it
//! (`[modules] source = "linked"`, `OPENHUMAN_MODULE_SOURCE=linked`, or a
//! per-module override). Everything else still loads from its release.
//!
//! # Which modules are available
//!
//! One cargo feature per module, named for its registry id, plus `all`. The
//! table is what the current vendor pins support:
//!
//! | feature          | status at the current vendor pins |
//! | ---------------- | --------------------------------- |
//! | `tinychannels`   | waits: v0.1.29 module crate pins `tinybus =0.1.4`, which cannot co-resolve with tinydocs-module (`=0.1.3`) and the workspace `tinybus` 0.1.3 |
//! | `tinydocs`       | linked (needs tinydocs v0.2.2, whose module crate pins the workspace `tinybus` 0.1.3) |
//! | `tinybox`        | waits: its module crate pins `tinybus =0.1.2`, which cannot unify with the workspace `tinybus` path |
//! | `tinyconnectors` | waits: its workspace pins a `tinybus` the host does not carry (0.1.2 at the pin, 0.1.4 upstream) |
//! | `tinyvoice`      | waits: upstream main needs nested `tinyagents` submodules that collide with the host's in Cargo.lock |
//! | `tinymcp`        | waits: `static-link` exists but `linked_module` is not exported |
//! | `tinycomputer`   | waits: no `static-link` |
//! | `tinyjuice`      | waits: v0.9.0 has `static-link` + `linked_module`, but needs `tinybus 0.1.4` (nested `vendor/tinybus`) while the workspace carries 0.1.3, so `LinkedFactory` types do not unify |
//! | `tinysearch`     | waits: v0.5.0 has `static-link` + `linked_module`, but needs `tinybus ^0.1.4` while its nested `vendor/tinybus` is 0.1.2 and the workspace carries 0.1.3, so Cargo cannot resolve the graph |
//! | `tinywallet`     | waits: v0.20.0 has `static-link` + `linked_module`, but its nested `vendor/tinybus` (cbb2eea) collides with the workspace `tinybus` (9062dd8) in Cargo.lock; also see the attestation note below |
//! | `tinyhosts`      | waits: no module crate yet |
//!
//! The waiting modules need the upstream static-link / tinybus alignment work (E0.3) and a re-pin;
//! each gets its feature here, and a line in [`linked_modules`], when that lands.
//!
//! # Attestation
//!
//! A linked module is attested by the executable's digest, not by a release
//! digest. Confidential members (tinywallet's attested calls) must accept that
//! digest in test builds for tinywallet to be linkable; until that is
//! confirmed, treat tinywallet as `source`-only for local testing.

use openhuman_rpc::embed::modules::{register_linked, LinkedFactory};

/// The modules this build links, as `(registry id, factory)`.
///
/// Empty when no module feature is enabled.
#[must_use]
pub fn linked_modules() -> Vec<(&'static str, LinkedFactory)> {
    let modules: Vec<(&'static str, LinkedFactory)> = vec![
        #[cfg(feature = "tinydocs")]
        ("tinydocs", tinydocs_module::linked_module),
    ];
    modules
}

/// Register every enabled module with the core's linked-module seam.
///
/// Call once at boot, before the first module load. Idempotent: a repeat call
/// replaces the same factories.
pub fn install() {
    for (id, factory) in linked_modules() {
        log::debug!("[modules-linked] registering id={id}");
        register_linked(id, factory);
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
