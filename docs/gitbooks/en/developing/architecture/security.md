---
description: >-
  The trust boundary for the autonomous core: the autonomy and risk policy,
  sandbox selection, the audit log, the encrypted secret store, the pairing
  guard and the redact() helper.
icon: shield-halved
---

# Security

`crates/openhuman-core/src/security/` is the trust boundary for the autonomous core. Look here first when you ask "is this agent action allowed, and if so, how is it confined?"

It owns:

- The autonomy and risk policy that decides whether a tool call is allowed.
- The sandbox selection that confines calls when the host supports it.
- The append-only audit log of every agent action.
- The encrypted secret store.
- The pairing guard that gates public binding of the RPC server.
- The `redact()` helper every other domain uses to keep plaintext credentials out of logs.

The decisions themselves are made by the TinySecurity native module (`vendor/tinysecurity`, pinned at v0.13.0, contract 1.20). The host links only its `tinysecurity-bus` contract, translates configuration into immutable module scopes, and maps the answers. A module that is missing, untrusted, faulted or slow denies; there is no host-side fallback decision.

It does not own the cross-domain `EncryptionEngine` (in `security/encryption/`) or per-channel credential storage (in `security/credentials/`).

## Public surface

| Item                                                                                                                          | File         | Purpose                                                                 |
| ----------------------------------------------------------------------------------------------------------------------------- | ------------ | ----------------------------------------------------------------------- |
| `SecurityPolicy`                                                                                                              | `policy/types.rs` (path checks in `policy/path_checks.rs`, the command gate's host client in `policy/native_commands.rs`, gating in `policy/enforcement.rs`) | Assembles runtime policy from `AutonomyConfig` + workspace dir.         |
| `AutonomyLevel` (`ReadOnly` / `Supervised` / `Full`)                                                                          | `policy/types.rs` | Three-step autonomy ladder.                                             |
| `ToolOperation`, `ActionTracker`                                                                                              | `policy/types.rs` | Operation kinds and per-session counting.                                 |
| `SecretStore`                                                                                                                 | `secret_storage/encrypted_store.rs` (`secrets.rs` re-exports it) | OS-keychain / encrypted-file secret persistence with round-trip helpers. |
| `HostAuditEvent`, `HostAuditKind`, `record`                                                                                   | `audit_record.rs` | Host-observed rows (a shell command's result) appended to the module's audit stream. |
| `PairingGuard`, `constant_time_eq`, `is_public_bind`                                                                          | `pairing.rs` (`PairingGuard` and `constant_time_eq` are re-exported from `tinychannels_bus::security`) | Pairing-token check before binding the RPC server publicly.             |
| `redact(value: &str) -> String`                                                                                               | `core.rs`    | Uniform 4-char-prefix redaction for logs.                               |
| `security_policy_info_for_config(&Config) -> Outcome<serde_json::Value>`                                                    | `ops.rs`     | RPC handler for the doctor / settings UI.                               |

## Policy gate

Every tool call passes `ToolPolicyGateMiddleware`, the outermost tool middleware, installed first by `agent/tinyagents/middleware/security_gate.rs::install` so a refused call never reaches another middleware. Turns, sub-agents, flows and the live voice harness all install it through that one function.

- Each verdict source answers `Allow`, `RequireApproval` or `Deny`. Several sources (the module, plus the channel's "ask a human first" policy when the session has one) combine as deny > require-approval > allow (`combine_decisions` in `middleware/tinysecurity.rs`).
- A module fault is a denial and is never retried.
- A required approval settles through the approval resolver and the approval gate, described below.

Host-tool admission is a question to the module, built from the tool the host will run and never from model arguments (`middleware/tinysecurity_module.rs`):

- `Check::HostAction` carries the tool's permission level and external effect. The module applies the tier check, the hourly action budget and, for a cron turn, the read-only rule. A tool that neither acts nor has an external effect is allowed without a bus call.
- `Check::HostShell` carries the command string and is checked through the per-subject command scope, the same scope that holds the hourly budget.
- Typed denials: `read_only_mode`, `rate_limited`, `scheduled_read_only` and `high_risk`, plus the floor, path, command and access categories.
- Cron turns run read-only: a scheduled turn may use read-only tools only, and anything else is denied with `scheduled_read_only`.

## Approvals

The approval engine is `security/approval/native/`, backed by the module (`EvaluateApproval2`, `Decide2`, `ConsumeApproval2`, `RecordExecution2`, `ExpireApproval2`). The module stores each request bound to the exact action and calls back into the host for its store, audit sink, prompt and human verification. The parking TTL is 10 minutes (`DEFAULT_APPROVAL_TTL` in `approval/gate.rs`). Timeouts and denials fail closed. See [Approval gate](../../features/approval-gate.md) for the user-facing behavior.

## Sandboxing

This module carries no sandbox backends. Per-session sandbox selection lives in `crates/openhuman-core/src/sandbox/`. It delegates local OS confinement and Docker execution to the TinyBox module. Shell command classification, the allowlist, the hidden-execution defence, the protected-path floor, the risk score and the hourly command budget belong to the TinySecurity module. `policy/native_commands.rs` translates the host settings into the module's immutable `CommandPolicy` and projects its answer back; nothing is classified in the host.

- **Every command is planned through the module.** Before a sandboxed spawn launches, `sandbox/spawn_plan/` registers a runtime scope (`RegisterSandboxScope2`), asks `Plan2` for the concrete operation, and reserves the single-use plan (`ConsumeSandboxPlan2`) immediately before the spawn. The spawn runs under the mounts, network and bounds of the plan. When the module denies or faults, the command does not run.
- **Native jail floor.** The native jail keeps a deny-read floor over the credential stores (`~/.ssh`, `~/.gnupg`, `~/.aws`, `~/.azure`, `~/.kube`, and Keychains on macOS). Grants that reach or contain a credential directory are dropped.
- **Untrusted text.** Text the host did not author reaches a shell only through `sandbox::untrusted::run`, which refuses a jail that is missing or not suitable for untrusted code. Scheduled shell jobs and hook commands run through it. `sandbox.status` reports `suitable_for_untrusted_code`, true only for a ready container or a ready kernel-isolated jail.
- **MCP stdio servers** start inside the TinyBox native jail through TinyMCP's `JailedSpawner`. Only servers the operator declares in `[mcp_client.servers]` carry `operator_trusted` and start as ordinary child processes; registry installs, and every server under SaaS, are jailed.
- **SaaS** ignores `OPENHUMAN_SANDBOX=off`. The host reports the switch to the security module, and the module plans a run on the host only for the trusted operator outside SaaS; it refuses hooks, scheduled jobs and registry MCP servers, which are planned onto a jail or container instead. The configured backend is a preference sent to the module, which picks the executor.

Current limitations, documented in `crates/openhuman-core/src/sandbox/README.md`:

- On macOS, Seatbelt reports best-effort enforcement, which the module does not accept as exact, so local sandboxing is refused there.
- A policy that allows the network is currently planned offline: the host has no surface to ask for the module's host-network approval, so sandboxed commands run without network.

Both are fixed in [tinysecurity#27](https://github.com/tinyhumansai/tinysecurity/pull/27), which is waiting on CI and is not yet pinned here.

## Autonomy ladder

`AutonomyLevel` is a three-step ladder that sets how strictly the policy gates tool calls. It only takes effect while `[autonomy] enabled = true`.

- `ReadOnly` blocks every writing class outright. No approval can authorize one.
- `Supervised` is the default. It allows reads and parks everything else for an approval round trip.
- `Full` allows reads and writes. It still parks the network, install and destructive classes.

The wire spellings are lowercase (`readonly`, `supervised`, `full`). The user-facing table is in [Approval gate](../../features/approval-gate.md).

`ToolOperation` names the kind of a tool call. `ActionTracker` keeps the per-session counts that the policy compares against its caps. The policy gate asks the module for a decision before every executable tool dispatch.

All of this applies only when the autonomy policy is on. `[autonomy] enabled = false` is the default, and `SecurityPolicy::from_config` carries that flag through every enforcement entry point. With the policy off, command classification, the approval gate, the command allowlist and the hourly action budget do nothing. `workspace_only`, `forbidden_paths` and the workspace-internal boundary are not enforced either.

One check never turns off. `is_always_forbidden` blocks credential stores (`~/.ssh`, `~/.gnupg`, `~/.aws`) and system roots. It also rejects `..` traversal and null bytes in a path. The module keeps this floor whatever the configuration (`DenialReason::Floor`), and the host repeats it as a synchronous preflight.

## Audit log

There is one security audit trail, and it is the TinySecurity module's: an ordered, contiguous stream per subject that the module sanitizes and the host persists. Approvals, automatic approvals, judge verdicts and sandbox plans are appended by the module. What only the host observes, a shell command's execution result, is recorded through the module's `RecordAudit2` (`audit_record.rs`). A row the module cannot record is logged at warn level; there is no second log file.

## Pairing guard

`PairingGuard` (in `pairing.rs`) stands between the RPC server and any attempt to bind to a non-loopback address. `is_public_bind` detects that case. `PairingGuard` then requires a pairing token, compared in constant time with `constant_time_eq`, before the bind is allowed. The iOS and LAN-companion pairing flow relies on this to keep an unpaired peer from attaching to the desktop core.

## Secret store

`SecretStore` (implemented in `security/secret_storage/encrypted_store.rs`, re-exported through `secrets.rs`) encrypts config-field secrets with ChaCha20-Poly1305 (`enc2:` prefix). It migrates the legacy XOR `enc:` format when it decrypts. Core owns the platform-independent secret-storage contracts, an in-memory default, and an optional encrypted-file backend. `openhuman-rpc::keyring` owns the native OS providers and installs them through those contracts before host boot. RPC product hosts explicitly select persistent storage; embed uses memory unless a provider or backend override is supplied. Backend selection and migration are described in `crates/openhuman-core/src/security/secret_storage/README.md`.

## `redact()`

`redact(value)` returns a uniform 4-character-prefix string (for example `"sk-a…"`) for use in logs and error messages. Use it whenever a secret, credential, token, or PII string is about to be formatted into a `log::` / `tracing::` call. Other domains call it directly: `credentials/`, `webhooks/`, `composio/`, the integration adapters.

## Layout

| Path                                                          | Role                                                                      |
| ------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `policy/` (`mod.rs`, `types.rs`, `path_checks.rs`, `command_checks.rs`, `enforcement.rs`, `policy_*_tests.rs`, `proptest_tests.rs`) | `SecurityPolicy`, `AutonomyLevel`, risk classification, path and command checks, action tracking. |
| `core.rs`, `core_tests.rs`                                    | `redact()` + small shared helpers.                                        |
| `audit_record.rs`                                             | Host rows for the module's audit stream.                                  |
| `secrets.rs`, `secret_storage/`                                      | `SecretStore` (implemented in `secret_storage/encrypted_store.rs`) + round-trip tests. |
| `pairing.rs`, `pairing_tests.rs`                              | `PairingGuard` + constant-time helpers.                                   |
| `ops.rs`                                                      | RPC handler (`security_policy_info_for_config`).                          |
| `schemas.rs`                                                  | Controller schemas + handler dispatch.                                    |
| `mod.rs`                                                      | Re-exports of the public surface above.                                   |
| `live_policy.rs`, `scrub.rs`, `tools.rs`                      | Live-policy lookup, the host scrubbing policy (`scrub::host_policy`), and the domain's own agent tools. |
| `approval/`, `credentials/`, `devices/`, `egress/`, `encryption/`, `keyring_consent/`, `prompt_injection/` | Sibling kernel-security domains under the same module. Each owns its own surface. This page covers the policy core only. |

Local PII detection belongs to [`tinymemory_api::safety`](https://github.com/tinyhumansai/tinymemory/blob/d1b06334faf7f8abb62bd1bd4ec4c4d4862f53a4/crates/tinymemory-api/src/safety/README.md) behind its `local-safety` feature. OpenHuman keeps privacy policy and redaction decisions.

## Calls into

- `crates/openhuman-core/src/config/`: `SecurityConfig`, `AutonomyConfig` for policy + sandbox selection.
- Workspace filesystem, for the audit stream's storage and the secret store.

## Called by

- `crates/openhuman-core/src/cron/ops.rs`: wraps shell jobs in `SecurityPolicy::from_config`.
- `crates/openhuman-core/src/tools/ops.rs` and `tools/impl/{browser,document,filesystem,network,presentation,system}/`: every executable tool consults `SecurityPolicy`.
- `crates/openhuman-core/src/tools/impl/network/{gate,mcp,mcp_server_tools}.rs`: risk-classify outbound calls.
- `crates/openhuman-core/src/memory/guard.rs`: wraps every memory write in `tinyredact::Policy::corroborated()`.
- `crates/openhuman-core/src/agent/tools/delegate.rs`: sub-agent dispatch goes through the autonomy gate.
- `crates/openhuman-core/src/security/credentials/`: uses `SecretStore` and `redact`.

## Tests

- Unit: `pairing_tests.rs`, `policy/policy_tests*.rs`, `policy/proptest_tests.rs`, `secret_storage/encrypted_store_tests*.rs`.
- `core_tests.rs` covers `redact()`.

## Related

- [`security/README.md`](https://github.com/tinyhumansai/openhuman/blob/main/crates/openhuman-core/src/security/README.md): the in-repo overview this page mirrors.
- [Architecture](../architecture.md): the wider system context.
- [Agent harness](agent-harness.md): where `SecurityPolicy` is consulted on every tool dispatch.
