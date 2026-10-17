use crate::core::runtime::DispatchCtx;

impl ApprovalGate {
    /// Run an engine call from synchronous code. The decision side of the gate is
    /// synchronous by contract (the embedding API, an agent's teardown, the
    /// harness's record hook), so the engine's async call runs on the shared
    /// blocking bridge instead of the caller's runtime, which cannot be blocked
    /// on a current-thread runtime.
    fn run_engine<T, F, Fut>(&self, op: F) -> anyhow::Result<T>
    where
        F: FnOnce(SharedEngine) -> Fut,
        Fut: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let future = op(Arc::clone(&self.engine));
        crate::storage::block_on_anyhow(async move { Ok(future.await) })
    }

    /// Write the *terminal* status of a tool call onto its approval
    /// audit row: how the call the gate reserved ended.
    ///
    /// Logs (but does not propagate) write errors: the tool has
    /// already run, so audit-log loss should never bubble up as a
    /// tool execution failure to the agent. If durable audit storage
    /// is required for compliance, callers wire it via a stronger
    /// guarantee than this best-effort hook.
    ///
    /// `error` is the (already sanitized) failure text of the call. The
    /// engine records the outcome class only; the text is never stored, which
    /// keeps whatever it quotes out of the durable log.
    pub fn record_execution(
        &self,
        request_id: &str,
        outcome: ExecutionOutcome,
        error: Option<&str>,
    ) {
        let id = request_id.to_owned();
        let recorded = self
            .run_engine(|engine| async move { engine.record_execution(&id, outcome).await })
            .unwrap_or_else(|bridge| Err(EngineError::Unavailable(bridge.to_string())));
        match recorded {
            Ok(()) => tracing::debug!(
                request_id = %request_id,
                outcome = outcome.as_str(),
                had_error = error.is_some(),
                "[approval::gate] recorded terminal execution"
            ),
            Err(EngineError::NotPending) => tracing::warn!(
                request_id = %request_id,
                outcome = outcome.as_str(),
                "[approval::gate] record_execution found no matching reserved request"
            ),
            Err(err) => tracing::error!(
                request_id = %request_id,
                outcome = outcome.as_str(),
                error = %err,
                "[approval::gate] record_execution write failed"
            ),
        }
    }

    /// Apply a user decision. Returns the now-decided
    /// [`PendingApproval`] when the request was parked in this process.
    ///
    /// `Ok(None)` means there is nothing to decide: the request was already
    /// decided, expired or cancelled, or was never parked here (an earlier launch
    /// left it; the engine cannot resume another instance's request).
    pub fn decide(
        &self,
        request_id: &str,
        decision: ApprovalDecision,
    ) -> anyhow::Result<Option<PendingApproval>> {
        let Some(route) = self.parked.request_routes.lock().get(request_id).cloned() else {
            return Ok(None);
        };
        // A removed agent's requests are denied by its teardown; a decision
        // racing it must not approve one.
        if route
            .approval_scope
            .as_deref()
            .is_some_and(super::ApprovalScope::is_closed)
        {
            return Ok(None);
        }
        if !matches!(
            decision,
            ApprovalDecision::ApproveOnce | ApprovalDecision::Deny
        ) && route.forced
        {
            anyhow::bail!("this action requires a one-time approval or denial");
        }
        let choice = if decision.is_approve() {
            Choice::Once
        } else {
            Choice::Deny
        };
        let id = request_id.to_owned();
        let label = decision.as_str();
        match self.run_engine(|engine| async move { engine.decide(&id, choice, label).await })? {
            Ok(_) => {}
            Err(EngineError::NotPending) => return Ok(None),
            Err(EngineError::Refused) => {
                anyhow::bail!("the approval engine refused the decision")
            }
            Err(EngineError::Unavailable(message)) => {
                anyhow::bail!("the approval engine could not apply the decision: {message}")
            }
        }
        // `ApproveAlwaysForTool` persistence (append to `autonomy.auto_approve`
        // + reload the live policy) is handled by the `approval_decide` RPC
        // handler, which is async and owns the config save+reload path. The
        // engine only records the decision for this one call; the next call
        // reads the saved list as its posture.
        if let Some(tx) = self.parked.take_waiter(request_id) {
            let _ = tx.send(decision);
        }
        // Routing (thread/client/tool_call_id) was recorded at park time —
        // see `Surface::deliver` — so a decision made after the in-memory
        // waiter already resolved (e.g. via the TTL/channel-drop paths in
        // `gate_intercept_decision.rs`) still reports `None` here, which is
        // correct: this fn only fires for a live `decide()` call.
        let route = self.parked.take_request_route(request_id).unwrap_or(route);
        BUS.publish(DomainEvent::ApprovalDecided {
            request_id: request_id.to_string(),
            tool_name: route.pending.tool_name.clone(),
            decision: decision.as_str().to_string(),
            thread_id: route.thread_id.clone(),
            client_id: route.client_id.clone(),
            tool_call_id: route.tool_call_id.clone(),
            resolution: None,
            agent_id: route.agent_id.clone(),
        });
        Ok(Some(route.pending))
    }

    /// [`Self::decide`] on behalf of `agent`: refuses with
    /// [`ApprovalError::WrongAgent`] when the request was parked by any other
    /// agent (or by the process itself), so one agent cannot answer another's
    /// approval.
    pub fn decide_for_agent(
        &self,
        agent: &str,
        request_id: &str,
        decision: ApprovalDecision,
    ) -> anyhow::Result<Option<PendingApproval>> {
        let owner = self
            .parked
            .request_routes
            .lock()
            .get(request_id)
            .map(|route| route.agent_id.clone());
        if let Some(owner) = owner {
            if owner.as_deref() != Some(agent) {
                tracing::warn!(
                    request_id = %request_id,
                    agent_id = %agent,
                    owner = owner.as_deref().unwrap_or("<process>"),
                    "[approval::gate] decision refused: request belongs to another agent"
                );
                return Err(ApprovalError::WrongAgent {
                    request_id: request_id.to_string(),
                }
                .into());
            }
        }
        self.decide(request_id, decision)
    }

    /// Deny every undecided request `agent` parked, resolving its waiters.
    /// Returns how many requests were denied.
    pub fn deny_all_for_agent(
        &self,
        _ctx: &DispatchCtx,
        agent: &str,
        resolution: &str,
    ) -> anyhow::Result<usize> {
        let request_ids: Vec<String> = self
            .parked
            .request_routes
            .lock()
            .iter()
            .filter(|(_, route)| route.agent_id.as_deref() == Some(agent))
            .map(|(id, _)| id.clone())
            .collect();
        let mut denied = 0;
        for request_id in request_ids {
            let Some(route) = self.parked.take_request_route(&request_id) else {
                continue;
            };
            let id = request_id.clone();
            match self.run_engine(|engine| async move { engine.cancel(&id).await }) {
                Ok(Ok(())) => {}
                Ok(Err(error)) => tracing::debug!(
                    request_id = %request_id,
                    error = %error,
                    "[approval::gate] the engine did not cancel a removed agent's request"
                ),
                Err(error) => tracing::debug!(
                    request_id = %request_id,
                    error = %error,
                    "[approval::gate] cancelling a removed agent's request could not run"
                ),
            }
            denied += 1;
            if let Some(tx) = self.parked.take_waiter(&request_id) {
                let _ = tx.send(ApprovalDecision::Deny);
            }
            if let Some(thread_key) = route.thread_key.as_deref() {
                self.parked
                    .clear_thread_route_if_owned(thread_key, &request_id);
            }
            BUS.publish(DomainEvent::ApprovalDecided {
                request_id: request_id.clone(),
                tool_name: route.pending.tool_name.clone(),
                decision: ApprovalDecision::Deny.as_str().to_string(),
                thread_id: route.thread_id.clone(),
                client_id: route.client_id.clone(),
                tool_call_id: route.tool_call_id.clone(),
                resolution: Some(resolution.to_string()),
                agent_id: Some(agent.to_string()),
            });
        }
        tracing::info!(
            agent_id = %agent,
            denied,
            resolution,
            "[approval::gate] denied every pending request of the agent"
        );
        Ok(denied)
    }

    /// Classify a [`Self::decide`] miss — i.e. when `decide` returned
    /// `Ok(None)` because the request is no longer pending here.
    ///
    /// - [`DecideMiss::AlreadyResolved`] — the request exists but was **already
    ///   decided, expired, cancelled, or closed for removal**.
    ///   This is the benign
    ///   double-tap / two-operator / expiry-while-live race the inline-approvals
    ///   design spec classifies as benign (TAURI-RUST-5EH).
    /// - [`DecideMiss::NeverRegistered`] — no request was ever stored for this
    ///   request_id. That is a genuine lost registration (a core restart dropped
    ///   the parked future before storing, or a stray id) and must stay a
    ///   Sentry signal.
    ///
    /// We disambiguate by asking the engine whether it stored the request and
    /// whether it left `Pending`. If the lookup itself errors we conservatively
    /// keep the event visible (`NeverRegistered`) rather than silently demoting.
    pub fn classify_decide_miss(&self, request_id: &str) -> DecideMiss {
        if self
            .request_scope(request_id)
            .is_some_and(|scope| scope.is_closed())
        {
            return DecideMiss::AlreadyResolved;
        }
        let id = request_id.to_owned();
        let known = self
            .run_engine(|engine| async move { engine.is_decided(&id).await })
            .map_err(|bridge| EngineError::Unavailable(bridge.to_string()))
            .and_then(|answer| answer);
        match known {
            Ok(Some(true)) => DecideMiss::AlreadyResolved,
            Ok(Some(false)) => DecideMiss::NeverRegistered,
            Ok(None) => DecideMiss::NeverRegistered,
            Err(err) => {
                tracing::warn!(
                    request_id = %request_id,
                    error = %err,
                    "[approval::gate] classify_decide_miss: lookup failed; treating as never-registered (keep visible)"
                );
                DecideMiss::NeverRegistered
            }
        }
    }

    /// The requests this tenant has parked and not yet seen decided, oldest
    /// first. A SaaS profile sees every agent's; anything else sees only its own
    /// agent's (the process, for the default orchestrator).
    pub fn list_pending(&self) -> anyhow::Result<Vec<PendingApproval>> {
        let tenant = super::gate::current_route_tenant();
        Ok(self.pending_where(|route| {
            route.tenant.profile == tenant.profile
                && (tenant.profile.is_some() || route.tenant.agent == tenant.agent)
        }))
    }

    /// [`Self::list_pending`] narrowed to `agent`'s requests; `None` lists the
    /// process's own.
    pub fn list_pending_for_agent(
        &self,
        ctx: &DispatchCtx,
        agent: Option<&str>,
    ) -> anyhow::Result<Vec<PendingApproval>> {
        let profile = ctx.tenant().ok().and_then(|tenant| tenant.profile);
        Ok(self.pending_where(|route| {
            route.tenant.profile == profile && route.pending.belongs_to(agent)
        }))
    }

    fn pending_where(&self, keep: impl Fn(&RequestRoute) -> bool) -> Vec<PendingApproval> {
        let mut rows: Vec<PendingApproval> = self
            .parked
            .request_routes
            .lock()
            .values()
            .filter(|route| keep(route))
            .map(|route| route.pending.clone())
            .collect();
        rows.sort_by_key(|row| row.created_at);
        rows
    }

    /// List recently decided requests for durable audit views.
    pub fn list_recent_decisions(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<super::types::ApprovalAuditEntry>> {
        let limit = limit.clamp(1, 500);
        let mut entries = self
            .run_engine(|engine| async move { engine.history(limit).await })?
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        entries.extend(store::list_recorded_decisions(&self.config, limit)?);
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.decided_at));
        entries.truncate(limit);
        Ok(entries)
    }

    /// List undecided requests correlated with a specific flow run (issue
    /// flow-approval-surface, PR2) — lets a dedicated Workflows review
    /// surface fetch just the gates blocking one run instead of filtering
    /// [`Self::list_pending`] client-side.
    pub fn list_pending_for_flow_run(
        &self,
        flow_id: &str,
        run_id: &str,
    ) -> anyhow::Result<Vec<PendingApproval>> {
        Ok(self
            .list_pending()?
            .into_iter()
            .filter(|row| {
                matches!(
                    &row.source_context,
                    Some(ApprovalSourceContext::Flow { flow_id: f, run_id: r, .. })
                        if f == flow_id && r == run_id
                )
            })
            .collect())
    }

    /// Grant "approve always for this flow" trust to `(flow_id, tool_name)`.
    /// Called by the `approval_decide` RPC handler after an
    /// [`ApprovalDecision::ApproveAlwaysForFlow`] decides a flow-origin row —
    /// mirrors the RPC-owns-persistence split documented on
    /// [`Self::decide`] for `ApproveAlwaysForTool`. The grant reaches the engine
    /// as the posture of the flow's next call.
    pub fn insert_flow_trust(&self, flow_id: &str, tool_name: &str) -> anyhow::Result<()> {
        store::insert_flow_trust(&self.config, flow_id, tool_name)
    }

    /// Whether `(flow_id, tool_name)` currently holds "approve always for
    /// this flow" trust. Exposed for tests and diagnostics.
    pub fn is_flow_tool_trusted(&self, flow_id: &str, tool_name: &str) -> anyhow::Result<bool> {
        store::is_flow_tool_trusted(&self.config, flow_id, tool_name)
    }

    /// Every `tool_name` currently trusted for `flow_id`, sorted. Consumed by
    /// `flows_approval_manifest` to diff the graph's required permissions
    /// against grants that already exist (re-save asks only for what's new).
    pub fn list_flow_trust(&self, flow_id: &str) -> anyhow::Result<Vec<String>> {
        store::list_flow_trust(&self.config, flow_id)
    }

    /// Revoke flow trust: all grants for `flow_id` when `tool_names` is
    /// `None` (flow deletion cleanup), or only the named grants. Returns the
    /// number of rows removed.
    pub fn delete_flow_trust(
        &self,
        flow_id: &str,
        tool_names: Option<&[String]>,
    ) -> anyhow::Result<usize> {
        store::delete_flow_trust(&self.config, flow_id, tool_names)
    }

    /// Write the durable audit record for one save-time pre-authorization
    /// grant (a born-decided `approve_always_for_flow` row) so blanket
    /// grants stay inspectable in Settings → Approval history.
    pub fn record_flow_preauthorization(
        &self,
        flow_id: &str,
        tool_name: &str,
    ) -> anyhow::Result<()> {
        store::record_flow_preauthorization(&self.config, flow_id, tool_name, &self.session_id)
    }

    /// Return the session id this gate was installed with (used by
    /// RPC handlers for diagnostics).
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Clone the instance barrier without retaining the routing-map lock.
    fn request_scope(&self, request_id: &str) -> Option<Arc<super::ApprovalScope>> {
        self.parked
            .request_routes
            .lock()
            .get(request_id)
            .and_then(|route| route.approval_scope.clone())
    }

    /// The request_id of the approval currently parked on `thread_id` by the
    /// ambient tenant (SaaS profile and/or embedded agent), if any. Used by
    /// the web channel to route an inbound yes/no reply to a decision.
    pub fn pending_for_thread(&self, thread_id: &str) -> Option<String> {
        self.pending_for_tenant_thread(&super::gate::current_route_tenant(), thread_id)
    }

    /// The request_id `agent` (`None`: the process) has parked on
    /// `thread_id` outside any SaaS profile, if any.
    pub fn pending_for_agent_thread(&self, agent: Option<&str>, thread_id: &str) -> Option<String> {
        let tenant = crate::core::runtime::Tenant {
            profile: None,
            agent: agent.map(str::to_owned),
        };
        self.pending_for_tenant_thread(&tenant, thread_id)
    }

    /// The request_id `tenant` has parked on `thread_id`, if any.
    pub fn pending_for_tenant_thread(
        &self,
        tenant: &crate::core::runtime::Tenant,
        thread_id: &str,
    ) -> Option<String> {
        self.parked
            .thread_to_request
            .lock()
            .get(&thread_route_key(tenant, thread_id))
            .cloned()
    }

    /// The full pending row parked on `thread_id`, if any.
    ///
    /// [`Self::pending_for_thread`] answers *which* request is parked; this
    /// answers *what it is*, which is what a UI needs to rebuild the approval
    /// card from scratch.
    ///
    /// The card reaches the UI as ONE fire-and-forget socket emit. If that emit
    /// misses — the addressed client's room is empty because the page reloaded,
    /// a rejoining socket was not yet in the thread room, or the bridge dropped
    /// the frame on broadcast lag — nothing re-sends it. The request stays
    /// parked server-side and the user is left with no card and no way to act,
    /// which is the whole failure: durable state delivered as an ephemeral
    /// event, with no way to reconcile the two. This lookup is that
    /// reconciliation, so a socket (re)joining a thread can be handed whatever
    /// is parked on it.
    ///
    /// Reads the routing map twice, so a request decided in between simply
    /// falls out as `None` rather than replaying a card the user has already
    /// answered.
    pub fn parked_request_for_thread(&self, thread_id: &str) -> Option<PendingApproval> {
        let request_id = self.pending_for_thread(thread_id)?;
        let row = self
            .parked
            .request_routes
            .lock()
            .get(&request_id)
            .map(|route| route.pending.clone())?;
        // Re-read the thread route before answering: between the two reads the
        // mapping can be cleared by a caller-bound abandon or overwritten by a
        // replacement turn parking a new approval on the same thread (#4774's
        // scenario). Either leaves this row pending while it is no longer the
        // thread's, and the only caller replays what it gets straight onto the
        // user's screen — so a stale row here is a stale approval card, not a
        // harmless read.
        //
        // Narrowing, not closing: the request itself could still be decided
        // after this check. That is the pre-existing fire-and-forget property of
        // the replay path, and `request_id` idempotency on the client is what
        // covers it (#6212 review).
        self.pending_for_thread(thread_id)
            .is_some_and(|current| current == request_id)
            .then_some(row)
    }

    /// Every approval still parked on `thread_id` by the ambient context's
    /// agent, oldest first.
    ///
    /// [`Self::parked_request_for_thread`] follows the single thread route,
    /// which holds only the newest park. Several async-delegated sub-agents can
    /// park on the same parent thread at once; each keeps its own waiter and
    /// [`RequestRoute`], so replay reads those instead and a rejoining client
    /// can rebuild every card, not only the last one raised.
    pub fn parked_requests_for_thread(&self, thread_id: &str) -> Vec<PendingApproval> {
        let agent = crate::core::runtime::agent_scope::current_agent_id();
        let mut rows: Vec<PendingApproval> = self
            .parked
            .request_routes
            .lock()
            .values()
            .filter(|route| {
                route.thread_id.as_deref() == Some(thread_id)
                    && route.agent_id.as_deref() == agent.as_deref()
            })
            .map(|route| route.pending.clone())
            .collect();
        rows.sort_by_key(|row| row.created_at);
        rows
    }

    /// Whether the parked `request_id` was routed to its chat thread from a
    /// detached task (an async-delegated sub-agent), so the replayed card keeps
    /// the live event's `detached` flag. `false` for an unknown request.
    pub fn request_is_detached(&self, request_id: &str) -> bool {
        self.parked
            .request_routes
            .lock()
            .get(request_id)
            .is_some_and(|route| route.detached)
    }
}
