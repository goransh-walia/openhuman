//! Host client for the module's scoped egress surface: `RegisterEgressPolicy`,
//! `ScopedUrlGuard`, `ScopedEgressCheck` and `ConsumeEgressAuthorization`, plus
//! the pinned HTTP executor that honours a grant.
//!
//! The module decides and issues a one-request [`TransportAuthorization`]; it
//! does not send. This executor is the transport the grant requires: it
//! consumes the grant once, connects only to the approved socket addresses
//! (no system DNS for that host, no ambient proxy, no pooled reuse), keeps the
//! original `Host`/SNI and hostname certificate checks, and never follows a
//! redirect. A redirect hop needs its own fresh [`guarded_send`]; a previous URL
//! string grants nothing. Any error denies; nothing falls back to an unchecked
//! request.

use std::time::Duration;

use tinysecurity_bus::{
    names, ConsumeEgressAuthorizationRequest, EgressDenial, EgressPolicy, EgressPolicyId,
    ScopedEgressRequest, ScopedEgressResponse, TransportAuthorization,
};

use crate::config::Config;

/// DNS resolution inside the module waits up to two seconds; allow the bus call
/// to outlast it so a slow resolver denies with `Resolver`, not a bus timeout.
const EGRESS_CALL_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const SEND_TIMEOUT: Duration = Duration::from_secs(60);

/// Result of a scoped destination check.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EgressOutcome {
    Authorized(Box<TransportAuthorization>),
    Denied(EgressDenial),
}

fn interpret(response: ScopedEgressResponse) -> Result<EgressOutcome, String> {
    if response.cacheable {
        return Err("TinySecurity egress response claimed to be cacheable".into());
    }
    match (response.authorization, response.denial) {
        (Some(auth), None) => Ok(EgressOutcome::Authorized(Box::new(auth))),
        (None, Some(denial)) => Ok(EgressOutcome::Denied(denial)),
        _ => Err("TinySecurity egress response was ambiguous".into()),
    }
}

async fn egress_proxy(config: &Config) -> Result<tinybus::Proxy, String> {
    super::require_enabled(config)?;
    Ok(super::scoped_client::module_proxy(config)
        .await?
        .with_timeout(EGRESS_CALL_TIMEOUT))
}

/// Register an immutable egress policy for a verified subject (trusted host
/// channel only).
pub(crate) async fn register_policy(
    config: &Config,
    policy: EgressPolicy,
) -> Result<EgressPolicyId, String> {
    let proxy = egress_proxy(config).await?;
    let id: EgressPolicyId = proxy
        .call_confidential(names::methods::REGISTER_EGRESS_POLICY, (policy,))
        .await
        .map_err(|_| "TinySecurity RegisterEgressPolicy failed")?;
    if id.0.is_empty() {
        return Err("TinySecurity returned an empty egress policy id".into());
    }
    Ok(id)
}

/// Authorize a generic agent HTTP(S) destination (never a local capability).
pub(crate) async fn scoped_url_guard(
    config: &Config,
    request: ScopedEgressRequest,
) -> Result<EgressOutcome, String> {
    let proxy = egress_proxy(config).await?;
    scoped_url_guard_on(&proxy, request).await
}

pub(crate) async fn scoped_url_guard_on(
    proxy: &tinybus::Proxy,
    request: ScopedEgressRequest,
) -> Result<EgressOutcome, String> {
    let response: ScopedEgressResponse = proxy
        .call_confidential(names::methods::SCOPED_URL_GUARD, (request,))
        .await
        .map_err(|_| "TinySecurity ScopedUrlGuard failed")?;
    interpret(response)
}

/// Authorize a destination with payload privacy and optional local capability.
pub(crate) async fn scoped_egress_check(
    config: &Config,
    request: ScopedEgressRequest,
) -> Result<EgressOutcome, String> {
    let proxy = egress_proxy(config).await?;
    scoped_egress_check_on(&proxy, request).await
}

pub(crate) async fn scoped_egress_check_on(
    proxy: &tinybus::Proxy,
    request: ScopedEgressRequest,
) -> Result<EgressOutcome, String> {
    let response: ScopedEgressResponse = proxy
        .call_confidential(names::methods::SCOPED_EGRESS_CHECK, (request,))
        .await
        .map_err(|_| "TinySecurity ScopedEgressCheck failed")?;
    interpret(response)
}

/// Consume the grant exactly once before sending. `false` and errors deny.
pub(crate) async fn consume_on(
    proxy: &tinybus::Proxy,
    request: ScopedEgressRequest,
    authorization: TransportAuthorization,
) -> Result<(), String> {
    let subject = request.subject.clone();
    let accepted: bool = proxy
        .call_confidential(
            names::methods::CONSUME_EGRESS_AUTHORIZATION,
            (ConsumeEgressAuthorizationRequest {
                request,
                subject,
                authorization,
            },),
        )
        .await
        .map_err(|_| "TinySecurity ConsumeEgressAuthorization failed")?;
    if accepted {
        Ok(())
    } else {
        Err("TinySecurity refused the egress authorization".into())
    }
}

/// Build a one-shot client that can reach only the approved sockets.
///
/// `resolve_to_addrs` bypasses system DNS for the grant's hostname while the
/// URL keeps the hostname, so `Host`, SNI and certificate verification use the
/// original name. Redirects, ambient proxies and idle pooling are off.
pub(crate) fn pinned_client(auth: &TransportAuthorization) -> Result<reqwest::Client, String> {
    if auth.addresses.is_empty() {
        return Err("egress authorization carries no approved address".into());
    }
    let url = url::Url::parse(&auth.url).map_err(|_| "egress authorization URL is invalid")?;
    let host_matches = url
        .host_str()
        .is_some_and(|host| host.eq_ignore_ascii_case(&auth.hostname));
    if !host_matches || url.port_or_known_default() != Some(auth.port) {
        return Err("egress authorization does not match its URL".into());
    }
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .pool_max_idle_per_host(0)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(SEND_TIMEOUT)
        .resolve_to_addrs(&auth.hostname, &auth.addresses)
        .build()
        .map_err(|_| "egress client construction failed".into())
}

/// Send `request` over a pinned connection to the grant's approved sockets.
pub(crate) async fn send_pinned(
    auth: &TransportAuthorization,
    request: &ScopedEgressRequest,
) -> Result<reqwest::Response, String> {
    let client = pinned_client(auth)?;
    let method = reqwest::Method::from_bytes(request.method.as_bytes())
        .map_err(|_| "egress method is invalid")?;
    let mut builder = client.request(method, &auth.url);
    for header in &request.headers {
        builder = builder.header(&header.name, &header.value);
    }
    if !request.payload.is_empty() {
        builder = builder.body(request.payload.clone());
    }
    builder.send().await.map_err(|error| {
        format!(
            "pinned egress request failed (connect={})",
            error.is_connect()
        )
    })
}

/// Check, consume and send one request. A 3xx answer is returned unfollowed:
/// the caller re-enters here with the next hop so every hop is checked afresh.
pub(crate) async fn guarded_send(
    config: &Config,
    request: ScopedEgressRequest,
) -> Result<reqwest::Response, String> {
    let proxy = egress_proxy(config).await?;
    let auth = match scoped_url_guard_on(&proxy, request.clone()).await? {
        EgressOutcome::Authorized(auth) => *auth,
        EgressOutcome::Denied(denial) => {
            log::debug!("[security][egress] denied reason={denial:?}");
            return Err(format!("egress denied: {denial:?}"));
        }
    };
    consume_on(&proxy, request.clone(), auth.clone()).await?;
    send_pinned(&auth, &request).await
}

// ---------------------------------------------------------------------------
// Agent tool destinations
// ---------------------------------------------------------------------------

/// The module-facing description of one network tool call: what the tool is
/// about to send, as the tool will send it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToolDestination<'a> {
    pub(crate) url: &'a str,
    pub(crate) method: &'a str,
    pub(crate) headers: &'a [(String, String)],
    pub(crate) body: Option<&'a str>,
}

/// Registered open-public policies, least recently used last, so one subject
/// keeps one policy for as long as its settings are unchanged.
fn policy_cache() -> &'static std::sync::Mutex<Vec<(EgressPolicy, EgressPolicyId)>> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Vec<(EgressPolicy, EgressPolicyId)>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

/// The open-public policy for `owner` under `privacy`: any `http(s)` destination
/// that passes the module's address floor and privacy checks. Nothing local is
/// registered, so a generic tool URL can never reach a host-local capability.
pub(crate) fn open_public_policy(
    owner: tinysecurity_bus::SubjectScope,
    privacy: tinysecurity_bus::PrivacyMode,
) -> EgressPolicy {
    EgressPolicy {
        owner,
        ceiling: tinysecurity_bus::AccessTier::Full,
        privacy,
        destinations: Vec::new(),
        data_categories: vec![tinysecurity_bus::DataCategory::UserContent],
        local_capabilities: Vec::new(),
        validity_ms: 30_000,
        open_public: true,
        preflight_transports: Vec::new(),
    }
}

/// The scoped request the module judges for `destination` under `policy_id`.
pub(crate) fn tool_request(
    policy_id: EgressPolicyId,
    owner: tinysecurity_bus::SubjectScope,
    destination: &ToolDestination<'_>,
) -> ScopedEgressRequest {
    ScopedEgressRequest {
        policy_id,
        subject: owner,
        tier: tinysecurity_bus::AccessTier::Full,
        url: destination.url.to_owned(),
        method: destination.method.to_ascii_uppercase(),
        // The URL, headers and body are model-supplied, so the host cannot call
        // them content-free metadata.
        category: tinysecurity_bus::DataCategory::UserContent,
        payload: destination.body.unwrap_or_default().to_owned(),
        headers: destination
            .headers
            .iter()
            .map(|(name, value)| tinysecurity_bus::EgressHeader {
                name: name.clone(),
                value: value.clone(),
            })
            .collect(),
        transport: tinysecurity_bus::EgressTransport::PinnedDirect,
        local_capability: None,
        transformation: None,
    }
}

async fn cached_policy_id(config: &Config, policy: EgressPolicy) -> Result<EgressPolicyId, String> {
    if let Ok(mut cache) = policy_cache().lock() {
        if let Some(id) = super::lru_lookup(&mut cache, &policy) {
            return Ok(id);
        }
    }
    let id = register_policy(config, policy.clone()).await?;
    if let Ok(mut cache) = policy_cache().lock() {
        super::lru_insert(&mut cache, policy, id.clone());
    }
    Ok(id)
}

/// Authorize the destination of one agent tool call and return the only socket
/// addresses it may connect to.
///
/// The subject is the authenticated tenant (never the request), the privacy
/// mode is the live one, and the grant is consumed here, immediately before the
/// tool connects, because the tool owns the connection. `Err` carries the
/// tool-facing refusal: a module that is missing, untrusted, faulted or slow
/// refuses, and so does any denial.
pub(crate) async fn authorize_tool_destination(
    workspace_dir: &std::path::Path,
    destination: &ToolDestination<'_>,
) -> Result<Vec<std::net::SocketAddr>, String> {
    let unavailable = || {
        format!(
            "{} TinySecurity egress authorization unavailable; request denied",
            tinytools::status::POLICY_BLOCKED_MARKER
        )
    };
    let saas = crate::core::runtime::is_saas();
    let config = super::config::host_config(workspace_dir, saas)
        .await
        .map_err(|error| {
            log::warn!("[security][egress] configuration unavailable: {error}");
            unavailable()
        })?;
    let tenant = crate::core::runtime::tenant::current_tenant().map_err(|_| unavailable())?;
    let owner = super::policy_translation::subject_scope(&tenant, workspace_dir, saas)
        .map_err(|error| {
            log::warn!("[security][egress] no verified subject: {error}");
            unavailable()
        })?;
    let privacy = super::policy_translation::privacy_mode(
        crate::security::live_policy::current_privacy_mode(),
    );
    let policy_id = cached_policy_id(&config, open_public_policy(owner.clone(), privacy))
        .await
        .map_err(|error| {
            log::warn!("[security][egress] policy registration failed: {error}");
            unavailable()
        })?;
    let request = tool_request(policy_id, owner, destination);
    let proxy = egress_proxy(&config).await.map_err(|_| unavailable())?;
    let auth = match scoped_url_guard_on(&proxy, request.clone()).await {
        Ok(EgressOutcome::Authorized(auth)) => *auth,
        Ok(EgressOutcome::Denied(denial)) => {
            log::debug!("[security][egress] tool destination denied reason={denial:?}");
            return Err(format!(
                "{} Destination refused by the security module ({denial:?}). Do not retry this URL.",
                tinytools::status::POLICY_BLOCKED_MARKER
            ));
        }
        Err(error) => {
            log::warn!("[security][egress] module unavailable; destination denied: {error}");
            return Err(unavailable());
        }
    };
    consume_on(&proxy, request, auth.clone())
        .await
        .map_err(|error| {
            log::warn!("[security][egress] grant not accepted: {error}");
            unavailable()
        })?;
    Ok(auth.addresses)
}

#[cfg(test)]
#[path = "security_egress_tests.rs"]
mod tests;
