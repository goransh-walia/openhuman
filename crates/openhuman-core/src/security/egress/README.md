# Security / Egress

The privacy epic's "egress spine" (#4436, #4441): a single, uniform answer to
"what leaves the device, to where, and why" for every external data transfer,
plus the `LocalOnly` enforcement chokepoint built on top of it.

## Key files

| File | Purpose |
| --- | --- |
| [`types.rs`](./types.rs) | `EgressDescriptor`, `DataKind`, `EgressReason`, `IdentificationRisk`: the descriptor contract |
| [`emit.rs`](./emit.rs) | `emit_external_transfer`, the single publish chokepoint, plus per-turn dedup |
| [`assess.rs`](./assess.rs) | `assess_payload`, `EgressDescriptor::assessed[_parts]`: fills the risk fields from the real outgoing payload via the module's `Redact` (Sensitive mode only) |
| [`enforce.rs`](./enforce.rs) | `local_only_blocks` (pure decision) and its two side-effecting wrappers, `enforce_egress` / `local_only_tool_block` |

## Public surface

- `EgressDescriptor`: `provider_slug` (destination, e.g. `"openai"`,
  `"composio"`, `"openhuman_backend"`, `"network"`), `service` (endpoint/model/
  host), `is_external` (false for local runtimes, never published),
  `reason: EgressReason`, `data_kinds: Vec<DataKind>`, and the S5 forward-compat
  fields `risk_level: IdentificationRisk` / `risk_categories: Vec<String>`
  (default `Unknown`/empty until the PII/identification-risk detector
  populates them). Built via semantic constructors,
  `EgressDescriptor::inference`, `::composio`, `::integration`, `::embedding`,
  `::network_fetch`, rather than the struct literal.
- `emit_external_transfer(descriptor)`: the one call every external-egress
  point makes right before a transfer leaves the device. It drops non-external
  transfers, attaches best-effort chat routing from the
  turn's `ApprovalChatContext` (an argument, or carried by the `EgressLedger`), and fire-and-forget publishes
  `DomainEvent::ExternalTransferPending` on `crate::core::bus::BUS`. This
  never blocks or fails the caller.
- `EgressLedger::per_turn()` / `emit_external_transfer_in(&ledger, d)`: an explicit per-turn dedup ledger, passed down the provider factory chain, so a single
  managed chat turn's multi-model fan-out (primary + workload-tier + summarizer
  builds) collapses repeated disclosures of the same destination into one
  event; every other call path emits unconditionally.
- `local_only_blocks(mode, desc) -> bool`: pure decision, blocks only when
  `mode == PrivacyMode::LocalOnly`, `desc.is_external`, and the transfer is not
  an exempt backend control-plane round-trip (`is_control_plane`, private).
  `Standard` and `Sensitive` never block here.
- `enforce_egress(desc) -> anyhow::Result<()>`: thin wrapper for
  `anyhow`-returning egress sites (composio, integrations, cloud embeddings);
  `Err` with a clean, non-sensitive message when blocked.
- `local_only_tool_block(desc) -> Option<String>`: thin wrapper for agent
  tools that return `Ok(ToolResult::error(..))` on denial; the message is
  prefixed with `POLICY_BLOCKED_MARKER` so the harness treats it as a
  permanent, non-retryable block.

Both `enforce_egress` and `local_only_tool_block` read the live mode via
`security::live_policy::current_privacy_mode`, which defaults to
`PrivacyMode::Standard` (no restriction) when no session policy is installed
(CLI / cron / background), so enforcement is a correct no-op there.

## Payload assessment (S5)

A descriptor is built where the destination is known; the payload exists only at
the send. Sites that hold the payload fill the risk fields with
`EgressDescriptor::assessed(payload)` / `assessed_parts(parts)`: the module's
`Redact` (`Text`) is asked whether it would change the payload, the same test its
own Sensitive egress rule applies. Unchanged is `IdentificationRisk::None`;
changed is `High` with category `credential_or_identifier`; no answer is
`Unknown`, which Sensitive refuses. Only `Sensitive` depends on the answer, so
under every other mode nothing is sent to the module and the risk stays
`Unknown`.

Sites: Composio execute (arguments), managed-backend round-trips (JSON body;
multipart uploads stay `Unknown`), cloud embeddings (inputs), media generation
(prompt; reference media stays `Unknown`), the network tools (the call's URL,
headers and body, assessed once per call by `RoutedGate`) and chat inference (the
outgoing messages, by `EgressAssessMiddleware` as a model wrap). The inference
destination is still disclosed once per turn when the model is built; the wrap
only enforces. A local primary with an external fallback route is not covered by
the wrap, which is installed when the primary leaves the device.

## Relates to

- `config/schema/privacy.rs`: `PrivacyMode` (`Standard` / `Sensitive` /
  `LocalOnly`), the config-facing enum `local_only_blocks` matches on.
- [`tinymemory_api::safety`](https://github.com/tinyhumansai/tinymemory/blob/d1b06334faf7f8abb62bd1bd4ec4c4d4862f53a4/crates/tinymemory-api/src/safety/README.md): the opt-in local identification-risk detector
  that will eventually populate `EgressDescriptor::risk_level` /
  `risk_categories` via `EgressDescriptor::with_risk`.
- `core/events.rs`: `DomainEvent::ExternalTransferPending { descriptor,
  thread_id, client_id }`, the event `emit_external_transfer` publishes.

## Used by

- `memory/guard/policy.rs` (`HostGuardPolicy`): memory-write egress guard.
- `integrations/client/requests.rs`, `integrations/composio/client/execute.rs`,
  `integrations/composio/execute_dispatch.rs`: backend and Composio calls.
- `tools/impl/network/{curl,http_request,web_fetch}.rs`: agent network-fetch
  tools.
- `search/tools/{tavily/client,exa}.rs`, `embeddings/cloud_adapter.rs`,
  `inference/provider/factory.rs` and its `factory/` submodules: search and
  cloud inference/embedding providers.
- `web_chat/event_bus.rs`: subscribes to `ExternalTransferPending` and
  bridges it to the `external_transfer_pending` socket event for the frontend.

## Tests

- [`types_tests.rs`](./types_tests.rs), [`emit_tests.rs`](./emit_tests.rs), [`enforce_tests.rs`](./enforce_tests.rs).

## Further reading

- [Parent module (`security`)](../README.md)
- [Privacy mode](../../../../../docs/gitbooks/en/features/privacy-mode.md)
- [Privacy and security](../../../../../docs/gitbooks/en/features/privacy-and-security.md)
- [Security architecture](../../../../../docs/gitbooks/en/developing/architecture/security.md)
