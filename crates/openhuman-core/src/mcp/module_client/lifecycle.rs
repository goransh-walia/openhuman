//! Configured-client cache admission, retirement and acknowledged cleanup.

use super::*;

impl McpConfiguredClient {
    /// The process-owned configured object for this workspace. Entries remain
    /// reachable until `close` confirms cleanup; workspace/config switching is
    /// serialized by the client itself.
    pub(crate) fn for_config(config: &Config) -> Result<Arc<Self>, ModuleCallError> {
        if SHUTTING_DOWN.load(Ordering::Acquire) {
            ModuleClient::report_unavailable(MODULE);
            return Err(ModuleCallError::Unavailable);
        }
        if !config.modules.enabled {
            ModuleClient::report_unavailable(MODULE);
            return Err(ModuleCallError::Unavailable);
        }
        if current_agent_context_is_removed() {
            ModuleClient::report_unavailable(MODULE);
            return Err(ModuleCallError::Unavailable);
        }
        let data_dir = crate::core::runtime::agent_scope_dir(config);
        if Self::cleanup_pending(&data_dir) {
            return Err(ModuleCallError::Unavailable);
        }
        #[cfg(test)]
        {
            let mut hook = AFTER_CONTEXT_CHECK
                .get_or_init(|| Mutex::new(None))
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if hook
                .as_ref()
                .is_some_and(|(expected, _)| expected == &data_dir)
            {
                if let Some((_, sender)) = hook.take() {
                    let _ = sender.send(());
                }
            }
        }
        let clients = CLIENTS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut clients = clients
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // The cleanup reservation can be installed after the first check but
        // before this cache lock is acquired. Recheck before creating a client.
        if current_agent_context_is_removed() || Self::cleanup_pending(&data_dir) {
            ModuleClient::report_unavailable(MODULE);
            return Err(ModuleCallError::Unavailable);
        }
        if let Some(client) = clients.get(&data_dir) {
            if client.retiring.load(Ordering::Acquire) {
                return Err(ModuleCallError::Unavailable);
            }
            return Ok(Arc::clone(client));
        }
        let client = Arc::new(Self::new(&data_dir, config)?);
        clients.insert(data_dir, Arc::clone(&client));
        Ok(client)
    }

    /// A snapshot of configured workspaces currently retained by the host.
    /// The supervisor only drains open objects; it never creates a workspace.
    pub(crate) fn open_clients() -> Vec<(PathBuf, Arc<Self>)> {
        CLIENTS
            .get()
            .map(|clients| {
                clients
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .iter()
                    .map(|(workspace, client)| (workspace.clone(), Arc::clone(client)))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Drain observations from the module-owned supervisor for an open child.
    /// An object that is opening or closing is skipped and will be revisited
    /// on the next host poll.
    pub(crate) async fn drain_supervisor_events(
        &self,
        limit: usize,
    ) -> Result<Option<SupervisorBatch>, ModuleCallError> {
        let state = self.state.lock().await;
        if SHUTTING_DOWN.load(Ordering::Acquire) || self.retiring.load(Ordering::Acquire) {
            return Err(ModuleCallError::Unavailable);
        }
        let State::Open(object) = &*state else {
            return Ok(None);
        };
        let args = serde_json::to_value((limit,)).map_err(|_| ModuleCallError::ModuleFault)?;
        let reply = self
            .port
            .call_object(
                &object.object_path,
                names::methods::DRAIN_SUPERVISOR_EVENTS,
                args,
                false,
            )
            .await?;
        serde_json::from_value(reply)
            .map(Some)
            .map_err(|_| ModuleCallError::ModuleFault)
    }

    /// Register this runtime's cleanup. The flag is reset by the drained hook,
    /// so a later embedded-runtime start can register a fresh one.
    pub(crate) fn register_shutdown_hook() {
        if SHUTDOWN_REGISTERED.swap(true, Ordering::AcqRel) {
            return;
        }
        SHUTTING_DOWN.store(false, Ordering::Release);
        crate::core::shutdown::register(|| async {
            SHUTTING_DOWN.store(true, Ordering::Release);
            let clients = CLIENTS
                .get()
                .map(|clients| {
                    clients
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .values()
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for client in clients {
                if client.retiring.load(Ordering::Acquire) {
                    let data_dir = PathBuf::from(&client.data_dir);
                    let reservation = Self::reservation_for(&data_dir);
                    Self::close_retired_client(data_dir, client, reservation).await;
                } else if let Err(error) = client.close().await {
                    tracing::warn!(?error, "[mcp] workspace cleanup is still pending");
                }
            }
            Self::retry_orphaned_cleanups();
            SHUTDOWN_REGISTERED.store(false, Ordering::Release);
        });
    }

    pub(crate) fn new(
        data_dir: &std::path::Path,
        config: &Config,
    ) -> Result<Self, ModuleCallError> {
        let data_dir = data_dir
            .to_str()
            .ok_or(ModuleCallError::IncompatibleContract)?
            .to_owned();
        Ok(Self::with_port(
            data_dir,
            Box::new(TinyMcpPort {
                root: ModuleClient::new(config.clone()),
            }),
        ))
    }

    #[cfg(test)]
    pub(crate) fn install_test_port(
        config: &Config,
        port: Box<dyn McpModulePort>,
    ) -> Result<Arc<Self>, ModuleCallError> {
        let data_dir = crate::core::runtime::agent_scope_dir(config);
        let client = Arc::new(Self::with_port(
            data_dir
                .to_str()
                .ok_or(ModuleCallError::IncompatibleContract)?
                .to_owned(),
            port,
        ));
        let clients = CLIENTS.get_or_init(|| Mutex::new(HashMap::new()));
        clients
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(data_dir, Arc::clone(&client));
        Ok(client)
    }

    pub(super) fn with_port(data_dir: String, port: Box<dyn McpModulePort>) -> Self {
        Self {
            data_dir,
            port,
            state: tokio::sync::Mutex::new(State::Closed),
            retiring: AtomicBool::new(false),
        }
    }

    /// Close an already-open agent-scoped client without creating one. A
    /// failed close remains in the cache, marked unavailable, so shutdown or
    /// a later cleanup attempt can retry the module's retained transition.
    pub(crate) async fn close_agent_data_dir(data_dir: &std::path::Path) -> bool {
        let key = data_dir.to_path_buf();
        let reservation = Self::reservation_for(&key);
        let Some(client) = Self::retire_cached_client(&key) else {
            return Self::finish_cleanup(&key, None, reservation);
        };
        if let Some(cleaned) = Self::finish_closed_client_synchronously(&key, &client, &reservation)
        {
            return cleaned;
        }
        Self::close_retired_client(key, client, reservation).await
    }

    /// Marks the exact cached client unavailable before scheduling cleanup.
    /// If no Tokio runtime is active, the retired client remains cached for the
    /// registered shutdown hook to close later.
    pub(crate) fn schedule_agent_close_data_dir(data_dir: &std::path::Path) {
        let key = data_dir.to_path_buf();
        let reservation = Self::reservation_for(&key);
        let Some(client) = Self::retire_cached_client(&key) else {
            Self::finish_cleanup(&key, None, reservation);
            return;
        };
        if Self::finish_closed_client_synchronously(&key, &client, &reservation).is_some() {
            return;
        }
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return;
        };
        runtime.spawn(async move {
            if !Self::close_retired_client(key, client, reservation).await {
                tracing::warn!("[mcp] agent client close remains pending for retry");
            }
        });
    }

    /// Retires the exact scoped client and runs `after_close` only after the
    /// module acknowledges its close. The path remains unavailable until the
    /// callback succeeds, so a replacement cannot reuse files being purged.
    pub(crate) fn schedule_agent_close_and_cleanup(
        data_dir: &std::path::Path,
        after_close: CleanupAction,
    ) {
        let key = data_dir.to_path_buf();
        let reservation = Self::reservation_with_action(&key, after_close);
        let Some(client) = Self::retire_cached_client(&key) else {
            if !Self::finish_cleanup(&key, None, reservation) {
                tracing::warn!("[mcp] agent cleanup remains pending for retry");
            }
            return;
        };
        if Self::finish_closed_client_synchronously(&key, &client, &reservation).is_some() {
            return;
        }
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return;
        };
        runtime.spawn(async move {
            if !Self::close_retired_client(key, client, reservation).await {
                tracing::warn!("[mcp] agent close and cleanup remain pending for retry");
            }
        });
    }

    pub(super) fn retry_orphaned_cleanups() {
        let reservations = CLEANUP_RESERVATIONS
            .get()
            .map(|reservations| {
                reservations
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .iter()
                    .map(|(path, reservation)| (path.clone(), Arc::clone(reservation)))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for (data_dir, reservation) in reservations {
            let has_client = CLIENTS.get().is_some_and(|clients| {
                clients
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .contains_key(&data_dir)
            });
            if !has_client {
                Self::finish_cleanup(&data_dir, None, reservation);
            }
        }
    }

    pub(crate) fn agent_close_pending(data_dir: &std::path::Path) -> bool {
        Self::cleanup_pending(data_dir)
            || CLIENTS
                .get()
                .and_then(|clients| {
                    clients
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .get(data_dir)
                        .map(|client| client.retiring.load(Ordering::Acquire))
                })
                .unwrap_or(false)
    }

    pub(super) fn cleanup_pending(data_dir: &std::path::Path) -> bool {
        CLEANUP_RESERVATIONS.get().is_some_and(|reservations| {
            reservations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .contains_key(data_dir)
        })
    }

    pub(super) fn reservation_for(data_dir: &std::path::Path) -> Arc<CleanupReservation> {
        let reservations = CLEANUP_RESERVATIONS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut reservations = reservations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Arc::clone(
            reservations
                .entry(data_dir.to_path_buf())
                .or_insert_with(|| Arc::new(CleanupReservation::default())),
        )
    }

    pub(super) fn reservation_with_action(
        data_dir: &std::path::Path,
        action: CleanupAction,
    ) -> Arc<CleanupReservation> {
        loop {
            let reservation = Self::reservation_for(data_dir);
            let mut after_close = reservation
                .after_close
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let current = CLEANUP_RESERVATIONS
                .get()
                .and_then(|reservations| {
                    reservations
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .get(data_dir)
                        .is_some_and(|stored| Arc::ptr_eq(stored, &reservation))
                        .then_some(())
                })
                .is_some();
            if !current {
                continue;
            }
            after_close.get_or_insert(action);
            return Arc::clone(&reservation);
        }
    }

    pub(super) fn retire_cached_client(data_dir: &std::path::Path) -> Option<Arc<Self>> {
        let clients = CLIENTS.get().map(|clients| {
            clients
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        })?;
        let client = clients.get(data_dir)?;
        client.retiring.store(true, Ordering::Release);
        Some(Arc::clone(client))
    }

    pub(super) fn finish_closed_client_synchronously(
        data_dir: &std::path::Path,
        client: &Arc<Self>,
        reservation: &Arc<CleanupReservation>,
    ) -> Option<bool> {
        let Ok(state) = client.state.try_lock() else {
            return None;
        };
        let is_closed = matches!(&*state, State::Closed);
        drop(state);
        is_closed.then(|| {
            Self::finish_cleanup(data_dir, Some(Arc::clone(client)), Arc::clone(reservation))
        })
    }

    pub(super) async fn close_retired_client(
        data_dir: PathBuf,
        client: Arc<Self>,
        reservation: Arc<CleanupReservation>,
    ) -> bool {
        match client.close().await {
            Ok(()) => Self::finish_cleanup(&data_dir, Some(client), reservation),
            Err(error) => {
                tracing::warn!(?error, "[mcp] agent client close remains pending");
                false
            }
        }
    }

    pub(super) fn finish_cleanup(
        data_dir: &std::path::Path,
        client: Option<Arc<Self>>,
        reservation: Arc<CleanupReservation>,
    ) -> bool {
        let after_close = reservation
            .after_close
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let reservations = CLEANUP_RESERVATIONS.get_or_init(|| Mutex::new(HashMap::new()));
        let is_current = reservations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(data_dir)
            .is_some_and(|stored| Arc::ptr_eq(stored, &reservation));
        if !is_current {
            return false;
        }
        if let Some(client) = client.as_ref() {
            let Some(clients) = CLIENTS.get() else {
                return false;
            };
            let clients = clients
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !clients
                .get(data_dir)
                .is_some_and(|current| Arc::ptr_eq(current, client))
            {
                return false;
            }
        }
        // The reservation still blocks admission for this directory while
        // the callback runs without the global client cache lock.
        if after_close.as_ref().is_some_and(|action| !action()) {
            return false;
        }
        if let Some(client) = client.as_ref() {
            let Some(clients) = CLIENTS.get() else {
                return false;
            };
            let mut clients = clients
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !clients
                .get(data_dir)
                .is_some_and(|current| Arc::ptr_eq(current, client))
            {
                return false;
            }
            clients.remove(data_dir);
        }
        let mut reservations = reservations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if reservations
            .get(data_dir)
            .is_some_and(|stored| Arc::ptr_eq(stored, &reservation))
        {
            reservations.remove(data_dir);
            true
        } else {
            false
        }
    }
}
