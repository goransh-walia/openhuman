impl ApprovalGate {
    /// Shared core of [`Self::intercept_audited`] and
    /// [`Self::intercept_audited_bounded`]. When `park_bound` is `Some` and
    /// shorter than the effective TTL, the park is capped at it; on that bound
    /// elapsing the park is abandoned cancellation-safely (waiter evicted,
    /// thread routing cleared, the request left open) and
    /// `*park_bound_elapsed` is set so the bounded caller can render its own
    /// fast-path result instead of a `Deny`.
    ///
    /// Returned boxed and `#[inline(never)]` on purpose: an `async fn` body is
    /// otherwise re-instantiated inside every crate / codegen unit that awaits
    /// it, and this state machine is large. Boxing here keeps one copy,
    /// compiled in this crate.
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    fn intercept_audited_inner<'a>(
        &'a self,
        turn: &'a ApprovalTurnContext,
        tool_name: &'a str,
        action_summary: &'a str,
        args_redacted: serde_json::Value,
        park_bound: Option<Duration>,
        park_bound_elapsed: &'a mut bool,
        tool_call_id: Option<&'a str>,
        mode: Mode,
    ) -> futures::future::BoxFuture<'a, (GateOutcome, Option<String>)> {
        Box::pin(self.intercept_audited_inner_body(
            turn,
            tool_name,
            action_summary,
            args_redacted,
            park_bound,
            park_bound_elapsed,
            tool_call_id,
            mode,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    async fn intercept_audited_inner_body(
        &self,
        turn: &ApprovalTurnContext,
        tool_name: &str,
        action_summary: &str,
        args_redacted: serde_json::Value,
        park_bound: Option<Duration>,
        park_bound_elapsed: &mut bool,
        tool_call_id: Option<&str>,
        mode: Mode,
    ) -> (GateOutcome, Option<String>) {
        let Mode { forced, track } = mode;
        // Origin tells us who scheduled this turn. Entry points (web channel,
        // channel runtime, cron, background jobs, CLI) scope a typed
        // `AgentTurnOrigin` around `run_turn`. Unlabelled callers map to
        // `Unknown`, which is denied — the gate refuses to execute an
        // external_effect tool from an unlabelled call site.
        // The gate has no context of its own yet (the approval-gate carrier is
        // a separate #7801 step), so it reads the task's turn scope once and
        // asks that for the origin, the agent's approval posture and its host
        // overrides.
        let turn_ctx = crate::core::runtime::DispatchCtx::ambient_turn();
        let origin = turn_ctx.turn_origin().unwrap_or(AgentTurnOrigin::Unknown);
        // Approval of job creation grants the scheduler permission to run the
        // saved prompt and deliver its answer, not permission for arbitrary
        // future external effects. Enforce this before global auto-approval,
        // per-tool allowlists, and per-agent approval bypasses.
        if let AgentTurnOrigin::TrustedAutomation {
            source: TrustedAutomationSource::Cron,
            job_id,
        } = &origin
        {
            tracing::warn!(
                tool = tool_name,
                job_id = %job_id,
                "[approval::gate] scheduled agent external effect denied"
            );
            return (
                GateOutcome::Deny {
                    reason: format!(
                        "{POLICY_DENIED_MARKER} Scheduled agent turns cannot use '{tool_name}' for an external effect."
                    ),
                },
                None,
            );
        }
        // SaaS has no per-user approval surface: never park, allow only what
        // the deployment's sandboxed tool groups open.
        if let Some(outcome) = saas_outcome_with(
            &crate::core::runtime::DispatchCtx::ambient(),
            crate::core::runtime::is_saas(),
            tool_name,
        ) {
            return (outcome, None);
        }
        if forced
            && !matches!(
                &origin,
                AgentTurnOrigin::WebChat { thread_id, client_id, .. }
                    if !thread_id.trim().is_empty() && !client_id.trim().is_empty()
            )
        {
            return (
                GateOutcome::Deny {
                    reason: format!(
                        "{POLICY_DENIED_MARKER} '{tool_name}' requires an interactive WebChat approval."
                    ),
                },
                None,
            );
        }
        tracing::debug!(
            tool = tool_name,
            ?origin,
            auto_approve_all = self.is_auto_approve_all_enabled(&turn_ctx),
            chat_context = turn.chat.is_some(),
            "[approval::gate] evaluating approval request"
        );

        // Branch by origin. Web chat parks for an in-app approval; an external
        // channel persists an audit row and parks (no routable approval surface
        // yet, so it TTL-denies); a workflow whose flow requires approval parks
        // for review; trusted automation (internal background jobs, pre-declared
        // workflow actions) and the CLI run through unchanged; Unknown fails
        // closed. Everything that parks goes to the engine, which also applies
        // the user's allowlist, per-flow trust and auto-approve setting as
        // audited automatic approvals.
        let (variant, flow_id, unlabelled) = match &origin {
            AgentTurnOrigin::WebChat { .. } => (
                if forced {
                    Variant::Forced
                } else {
                    Variant::Standard
                },
                None,
                false,
            ),
            AgentTurnOrigin::ExternalChannel {
                channel,
                sender,
                reply_target,
                message_id,
                ..
            } => {
                tracing::info!(
                    tool = tool_name,
                    channel = %channel,
                    sender = %sender.as_deref().unwrap_or("<unknown>"),
                    reply_target = %reply_target,
                    message_id = %message_id,
                    "[approval::gate] external channel turn — recording the request and parking"
                );
                // We do NOT short-circuit to Allow here — remote inputs are
                // untrusted. Without a routable surface the park TTL-denies; a
                // decision can still arrive via the thread card before the TTL.
                (Variant::Standard, None, false)
            }
            AgentTurnOrigin::TrustedAutomation {
                source: TrustedAutomationSource::Cron,
                ..
            } => {
                return (
                    GateOutcome::Deny {
                        reason: format!(
                            "{POLICY_DENIED_MARKER} Scheduled agent turns cannot use '{tool_name}' for an external effect."
                        ),
                    },
                    None,
                );
            }
            AgentTurnOrigin::TrustedAutomation {
                source: TrustedAutomationSource::Background,
                job_id,
            } => {
                tracing::debug!(
                    tool = tool_name,
                    job_id = %job_id,
                    "[approval::gate] trusted internal background job — allowing without prompt"
                );
                return (GateOutcome::Allow, None);
            }
            AgentTurnOrigin::TrustedAutomation {
                source:
                    TrustedAutomationSource::Workflow {
                        require_approval: false,
                    },
                job_id,
            } => {
                tracing::debug!(
                    tool = tool_name,
                    flow_id = %job_id,
                    "[approval::gate] trusted workflow automation — pre-declared action, \
                     allowing without prompt"
                );
                return (GateOutcome::Allow, None);
            }
            AgentTurnOrigin::TrustedAutomation {
                source:
                    TrustedAutomationSource::Workflow {
                        require_approval: true,
                    },
                job_id,
            } => {
                tracing::info!(
                    tool = tool_name,
                    flow_id = %job_id,
                    "[approval::gate] workflow run has require_approval enabled — parking for \
                     HITL review instead of auto-allowing the trust root"
                );
                // The engine does not apply the global tool allowlist to this
                // posture, so a flow the user forced to be reviewed stays
                // reviewed; per-flow trust and auto-approve still do.
                (Variant::FlowStrict, Some(job_id.clone()), false)
            }
            // Same trust decision for both: a local operator invoking the core
            // directly. They differ only in whether the turn's text was written
            // by a person (`turn_origin::is_user_authored`), which this gate
            // does not ask. Kept as one arm so the two can never drift apart on
            // the trust axis, which is the axis this gate owns.
            AgentTurnOrigin::Cli | AgentTurnOrigin::DirectChat => {
                tracing::debug!(
                    tool = tool_name,
                    origin = %origin.class(),
                    "[approval::gate] CLI / sub-agent caller — allowing without prompt"
                );
                return (GateOutcome::Allow, None);
            }
            // The one thing that has always approved a call from an unlabelled
            // site is the user's own "Always allow" for that exact tool. Nothing
            // else does: not auto-approve-everything, not an agent's approvals
            // being off, and nobody is asked.
            AgentTurnOrigin::Unknown
                if !forced
                    && self
                        .live_auto_approve(&turn_ctx)
                        .iter()
                        .any(|allowed| allowed == tool_name) =>
            {
                tracing::debug!(
                    tool = tool_name,
                    "[approval::gate] unlabelled call to a tool on the user's Always-allow list"
                );
                (Variant::Standard, None, true)
            }
            AgentTurnOrigin::Unknown => {
                tracing::warn!(
                    tool = tool_name,
                    "[approval::gate] agent turn has no origin label — refusing to execute \
                     external_effect tool from unlabelled call site"
                );
                return (
                    GateOutcome::Deny {
                        reason: format!(
                            "{POLICY_DENIED_MARKER} '{tool_name}' was blocked because this agent \
                             turn is missing its origin label, so the approval gate cannot decide \
                             who requested the action. Scheduling and other external-effect tools \
                             (e.g. cron_add / cron_update) are refused when the turn has no origin. \
                             This is an internal wiring gap, not something you did — the work most \
                             likely ran on a background task that did not carry the turn's origin \
                             forward; retry from a normal chat turn, or report it so the spawn site \
                             can be fixed."
                        ),
                    },
                    None,
                );
            }
        };

        // Chat context (thread/client id) for routing the yes/no reply — handed
        // to the turn by the web channel; absent for non-chat callers.
        //
        // Fallback (#5499): when the turn has no chat context but is
        // `WebChat`, route via the thread/client the origin itself carries. The
        // web channel builds the chat context and the `WebChat`
        // origin from the *same* thread_id/client_id (`web_chat::start_chat`),
        // so the two are identical whenever both are present. They diverge only
        // when a turn is detached from the chat turn that started it and keeps
        // the origin but not the approval context — most importantly
        // an async-delegated sub-agent (`spawn_async_subagent`, reached when the
        // orchestrator routes "remind me…" to `scheduler_agent`): the origin
        // travels but the chat context does not (`detached_child`). Without the fallback the gate
        // parks with `thread_id: None`, the web-channel surface drops the
        // `ApprovalRequested` event ("thread/client absent — NOT surfacing"),
        // and the park silently TTL-denies — so a `cron_add` scheduled from a
        // chat turn never completes.
        // Forced approvals route from the validated WebChat origin itself. A
        // stray chat context must not redirect the decision to another thread.
        let chat_ctx = (!forced).then(|| turn.chat.clone()).flatten();
        let origin_chat_route = match &origin {
            AgentTurnOrigin::WebChat {
                thread_id,
                client_id,
                ..
            } => Some((thread_id.clone(), client_id.clone())),
            _ => None,
        };
        // The fallback is taken exactly when the park runs on a task detached
        // from the chat turn that shows it — an async-delegated sub-agent. Such
        // a park can outlive that turn (the parent replies "I've asked the
        // image agent…" and its `chat_done` fires while the child still waits
        // here), so the route records it as `detached` and the web surface
        // (`is_detached_request`) marks the card: the frontend
        // keeps it across the parent's turn end instead of clearing it as a
        // park that "cannot outlive its turn" — the clear that left every
        // async `image_agent` / `video_agent` approval invisible until it
        // expired at 600s.
        let detached = !forced && chat_ctx.is_none() && origin_chat_route.is_some();
        if detached {
            tracing::debug!(
                tool = tool_name,
                "[approval::gate] no chat context on a WebChat turn — routing the \
                 approval via the origin's thread/client (async-delegated sub-agent path, #5499); \
                 marking the card detached so it survives the parent turn's end"
            );
        }
        // Any park raised inside a sub-agent run — detached or inline — gets
        // the shorter `SUBAGENT_APPROVAL_TTL` window.
        let subagent_park = !forced
            && (detached
                || tinyagents_harness::context::current_run_depth()
                    .is_some_and(|facts| facts.depth > 0));
        let chat_thread_id = chat_ctx
            .as_ref()
            .map(|c| c.thread_id.clone())
            .or_else(|| origin_chat_route.as_ref().map(|(t, _)| t.clone()));
        let chat_client_id = chat_ctx
            .as_ref()
            .map(|c| c.client_id.clone())
            .or_else(|| origin_chat_route.as_ref().map(|(_, c)| c.clone()));

        // Copilot-streaming flag — set by `flows::ops::flows_build` on the
        // streaming turn. It clamps the park window to `COPILOT_APPROVAL_TTL`;
        // see `ApprovalTurnContext::copilot_stream`.
        let copilot_stream = !forced && turn.copilot_stream;

        // Routes key on the tenant (profile and agent), so two SaaS profiles'
        // default agents, which carry no agent id, never share one.
        let route_tenant = super::gate::current_route_tenant();
        let agent_id = route_tenant.agent.clone();
        let thread_key = chat_thread_id
            .as_deref()
            .map(|thread_id| thread_route_key(&route_tenant, thread_id));
        // Resolve the clamped park TTL up front and hand it to the engine, so
        // the stored expiry and the actual wait use the same value — see
        // `Self::resolve_park_ttl` and the COPILOT_APPROVAL_TTL clamp. The engine
        // only ever shortens it.
        let effective_ttl =
            Self::resolve_park_ttl(self.effective_ttl(), copilot_stream, subagent_park);

        // Correlation context (flow-approval-surface, PR2): a Workflow-origin
        // park carries the flow id on the origin itself, but not the run id —
        // that comes from the turn's `flow_run`, which
        // `flows::ops::flows_run`/`flows_resume` set alongside the origin.
        // It is `None` for every non-flow caller (chat, cron,
        // background jobs, CLI, and even a Workflow origin reached without the
        // flows module's context, which "should never happen" but must not
        // panic), so `source_context` stays `None` there — unchanged chat
        // behavior.
        let source_context = match &origin {
            AgentTurnOrigin::TrustedAutomation {
                source: TrustedAutomationSource::Workflow { .. },
                job_id: flow_id,
            } => turn
                .flow_run
                .as_ref()
                .map(|ctx| ApprovalSourceContext::Flow {
                    flow_id: flow_id.clone(),
                    run_id: ctx.run_id.clone(),
                    node_id: None,
                }),
            _ => None,
        };

        let scope = turn_ctx
            .host_overrides()
            .and_then(|overrides| overrides.approval_scope());
        let (waiter, decision) = oneshot::channel::<ApprovalDecision>();
        let surface = Arc::new(Surface {
            parked: Arc::clone(&self.parked),
            context: ParkContext {
                chat_thread_id: chat_thread_id.clone(),
                chat_client_id,
                tool_call_id: tool_call_id.map(str::to_string),
                forced,
                agent_id,
                approval_scope: scope.clone(),
                thread_key: thread_key.clone(),
                detached,
                tenant: route_tenant.clone(),
                source_context,
            },
            waiter: Mutex::new(Some(waiter)),
            delivered: Mutex::new(None),
        });
        // Armed before the engine is asked: it delivers the prompt while the
        // check is still in flight, and a turn torn down right then must not
        // leave the waiter and routing behind.
        let mut guard = CancelGuard {
            engine: Arc::clone(&self.engine),
            parked: Arc::clone(&self.parked),
            surface: Arc::clone(&surface),
            armed: true,
        };
        let input = CheckInput {
            tenant: route_tenant,
            variant,
            posture: if unlabelled {
                // Only the user's tool grant stands; nothing else is configured
                // for a call nobody labelled, and the grant is what approves it.
                Posture {
                    require_approval: true,
                    tool_grants: vec![tool_name.to_string()],
                    ..Posture::default()
                }
            } else {
                self.posture(&turn_ctx, variant)
            },
            call_id: fresh_call_id(),
            tool: tool_name.to_string(),
            args: object_arguments(args_redacted),
            summary: action_summary.to_string(),
            thread: chat_thread_id,
            flow: flow_id,
            origin: if copilot_stream {
                CallOrigin::Copilot
            } else if subagent_park {
                CallOrigin::SubAgent
            } else {
                CallOrigin::Operator
            },
            ttl: effective_ttl,
            prompt: surface,
        };
        let checked = match self.engine.check(input).await {
            Ok(checked) => checked,
            Err(error) => {
                // The guard tears down whatever the engine delivered.
                tracing::error!(
                    tool = tool_name,
                    error = %error,
                    "[approval::gate] the approval engine could not decide — failing closed"
                );
                return (
                    GateOutcome::Deny {
                        reason: format!(
                            "{POLICY_DENIED_MARKER} The approval engine could not evaluate '{tool_name}' — denying for safety: {error}"
                        ),
                    },
                    None,
                );
            }
        };
        match checked {
            Checked::Approved { request_id } => {
                guard.disarm();
                tracing::debug!(
                    request_id = %request_id,
                    tool = tool_name,
                    "[approval::gate] approved by the user's configuration, skipping prompt"
                );
                self.reserved(request_id, tool_name, scope.as_deref(), track).await
            }
            Checked::Pending {
                request_id,
                expires_at_ms,
            } => {
                tracing::info!(
                    request_id = %request_id,
                    tool = tool_name,
                    "[approval::gate] tool call parked, waiting for decision"
                );
                // The engine's own expiry is the park window.
                let window = Duration::from_millis(expires_at_ms.saturating_sub(now_ms()))
                    .min(effective_ttl);
                if copilot_stream {
                    tracing::debug!(
                        tool = tool_name,
                        ttl_secs = COPILOT_APPROVAL_TTL.as_secs(),
                        "[approval::gate] flows_build copilot-streaming park — clamping park window to \
                         COPILOT_APPROVAL_TTL"
                    );
                }
                if subagent_park {
                    tracing::debug!(
                        tool = tool_name,
                        detached,
                        ttl_secs = window.as_secs(),
                        "[approval::gate] sub-agent park — clamping park window to SUBAGENT_APPROVAL_TTL"
                    );
                }
                // Optional caller-supplied park bound (issue #4756). A caller
                // (`composio_connect`) can cap how long the gate parks so a turn
                // degrades to a fast prompt instead of blocking to the full TTL.
                // Bounding must never *extend* the park, so we wait `min(bound, ttl)`;
                // the caller-bound abandon path fires only when the bound is what
                // elapses (`park_bound_active`).
                let park_bound_active = matches!(park_bound, Some(b) if b < window);
                let wait = match park_bound {
                    Some(b) => b.min(window),
                    None => window,
                };
                // The full timeout-vs-decide-vs-park-bound resolution lives in
                // `resolve_park` (gate_intercept_decision.rs), split out
                // purely to keep this file under the repo's per-file line budget.
                let outcome = self
                    .resolve_park(
                        &request_id,
                        tool_name,
                        wait,
                        decision,
                        park_bound_active,
                        park_bound_elapsed,
                        window,
                        scope.as_deref(),
                        track,
                    )
                    .await;
                // Reached only on a normal park resolution: `resolve_park` already
                // ran the exact teardown for its outcome, so disarm the RAII guard
                // (its Drop is reserved for external cancellation — see
                // `CancelGuard`).
                guard.disarm();
                // The routing mappings are only needed while parked; clear them on
                // every exit (decision, channel drop, or timeout).
                self.parked.clear_thread(&thread_key, &request_id);
                outcome
            }
        }
    }

    /// Reserve an approved request for execution, exactly once. The reservation
    /// is what authorizes the call to run: an agent removed meanwhile (its scope
    /// closed) or a request the engine no longer holds approved is denied.
    async fn reserved(
        &self,
        request_id: String,
        tool_name: &str,
        scope: Option<&super::ApprovalScope>,
        track: bool,
    ) -> (GateOutcome, Option<String>) {
        if scope.is_some_and(super::ApprovalScope::is_closed) {
            tracing::warn!(
                request_id = %request_id,
                tool = tool_name,
                "[approval::gate] the agent was removed before the approved call ran — cancelling"
            );
            let _ = self.engine.cancel(&request_id).await;
            return (
                GateOutcome::Deny {
                    reason: format!(
                        "{POLICY_DENIED_MARKER} The agent that asked to run '{tool_name}' was removed before it could run."
                    ),
                },
                None,
            );
        }
        if !track {
            // The caller cannot report how the call ends, so no reservation is
            // taken that would never be closed; the approval stays as granted.
            self.engine.release(&request_id);
            return (GateOutcome::Allow, None);
        }
        match self.engine.reserve(&request_id).await {
            Ok(()) => (GateOutcome::Allow, Some(request_id)),
            Err(error) => {
                tracing::error!(
                    request_id = %request_id,
                    tool = tool_name,
                    error = %error,
                    "[approval::gate] the approved call could not be reserved — failing closed"
                );
                (
                    GateOutcome::Deny {
                        reason: format!(
                            "{POLICY_DENIED_MARKER} The approval for '{tool_name}' could not be reserved for execution — denying for safety: {error}"
                        ),
                    },
                    None,
                )
            }
        }
    }
}

/// Largest argument document handed to the engine, in bytes. The module refuses
/// more than 32 KiB; what the host redacts is far smaller, and anything larger is
/// stood in for by its size rather than failing the whole call.
const MAX_ARGUMENT_BYTES: usize = 30_000;

/// The arguments as the engine takes them: always a JSON object of bounded size.
fn object_arguments(value: serde_json::Value) -> serde_json::Value {
    let object = match value {
        serde_json::Value::Object(_) => value,
        serde_json::Value::Null => serde_json::json!({}),
        other => serde_json::json!({ "value": other }),
    };
    let bytes = object.to_string().len();
    if bytes > MAX_ARGUMENT_BYTES {
        return serde_json::json!({ "truncated": true, "bytes": bytes });
    }
    object
}
