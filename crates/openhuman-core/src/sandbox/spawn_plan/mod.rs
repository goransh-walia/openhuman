//! Sandbox planning through the TinySecurity module (issue #7328, phase 5).
//!
//! Before the host launches a sandboxed command it asks the module what may run.
//! The module owns the decision (the isolation floor, executor selection,
//! filesystem and network ceilings, resource bounds, mandatory SaaS isolation);
//! the host supplies the facts, the concrete operation, the audit correlation, and
//! executes exactly the plan it gets back:
//!
//! 1. [`facts::scope`] turns the policy, the configuration and the executors TinyBox
//!    verified into immutable scope facts. A scope's id digests its facts, so a spawn
//!    with other facts gets its own scope.
//! 2. [`registry`] makes the module hold that scope for the subject
//!    (`RegisterSandboxScope2`) on the subject's own audit stream.
//! 3. [`plan`] sends `Plan2` for the concrete [`SpawnOperation`] and maps the issued
//!    artifact through [`BoxPlan::from_artifact`].
//! 4. [`consume`] reserves the artifact once, immediately before the spawn
//!    (`ConsumeSandboxPlan2`); a second use, or any change to the operation, is refused.
//!
//! Every failure is a refusal: a module fault, a denial, a malformed answer and an
//! artifact the host cannot map all mean the command does not run. Nothing falls
//! back to a host-side decision.

mod executors;
mod facts;
mod operation;
mod registry;

use std::path::Path;

use tinysecurity_bus::{
    ConsumeSandboxPlanRequest2, NetworkPolicy, PathGrant, ResourceLimits2, SandboxArtifact2,
    SandboxDenial2, SandboxPlanRequest2, SubjectScope, Verdict,
};

pub(crate) use facts::SpawnSource;
pub(crate) use operation::SpawnOperation;

use super::plan::{BoxPlan, PlanMappingError, PlannedExecutor};
use super::types::{SandboxBackendKind, SandboxPolicy};
use crate::config::Config;
use crate::modules::security::sandbox as client;
use crate::security::approval::native::audit_stream::AuditStream;

/// Why a spawn was not planned. Either way the command must not run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PlanError {
    /// The module planned and refused.
    Refused(SandboxDenial2),
    /// The module could not be asked, answered something unusable, or the plan
    /// cannot be applied by the executors the host drives.
    Unavailable(String),
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(denial) => write!(f, "the sandbox plan was refused ({denial:?})"),
            Self::Unavailable(reason) => write!(f, "the sandbox plan is unavailable: {reason}"),
        }
    }
}

impl std::error::Error for PlanError {}

fn unavailable(reason: impl Into<String>) -> PlanError {
    PlanError::Unavailable(reason.into())
}

/// A spawn the module planned and the host has not launched yet.
#[derive(Debug, Clone)]
pub(crate) struct PlannedSpawn {
    /// What TinyBox is asked to enforce: executor, grants, network and bounds.
    pub(crate) plan: BoxPlan,
    artifact: SandboxArtifact2,
    request: SandboxPlanRequest2,
    subject: SubjectScope,
    stream: AuditStream,
    proxy: tinybus::Proxy,
}

/// Where the module's bus proxy comes from.
#[derive(Clone, Copy)]
pub(crate) enum ModuleSource<'a> {
    /// The attested, pinned production module, loaded on demand.
    Production,
    /// A module a test loaded itself, with the executor facts it declares.
    Fixed(&'a FixedModule),
}

/// A security module and executor facts supplied by a test.
pub(crate) struct FixedModule {
    pub(crate) proxy: tinybus::Proxy,
    pub(crate) executor: tinysecurity_bus::SandboxExecutor2,
}

impl ModuleSource<'_> {
    async fn proxy(self, config: &Config) -> Result<tinybus::Proxy, PlanError> {
        match self {
            Self::Production => client::production_proxy(config).await.map_err(unavailable),
            Self::Fixed(fixed) => Ok(fixed.proxy.clone()),
        }
    }
}

/// What a spawn is planned from.
#[derive(Clone, Copy)]
pub(crate) struct SpawnRequest<'a> {
    pub(crate) config: &'a Config,
    /// The resolved policy: which sandbox backend, workspace, mounts and network.
    pub(crate) policy: &'a SandboxPolicy,
    pub(crate) source: SpawnSource,
    pub(crate) operation: &'a SpawnOperation<'a>,
    /// The operator turned the sandbox off for this process (never under SaaS).
    pub(crate) host_off: bool,
    pub(crate) module: ModuleSource<'a>,
}

/// The requested resource bounds, from the policy's Docker overrides.
fn limits(policy: &SandboxPolicy) -> ResourceLimits2 {
    let Some(overrides) = &policy.docker_overrides else {
        return ResourceLimits2::default();
    };
    ResourceLimits2 {
        cpu_time_ms: None,
        // Fractional CPUs become thousandths of a core, rounded up so a bound is
        // never silently loosened; absent bounds stay absent.
        cpu_millicores: overrides
            .cpu_limit
            .filter(|cpus| cpus.is_finite() && *cpus > 0.0)
            .map(|cpus| (cpus * 1000.0).ceil().min(f64::from(u32::MAX)) as u32),
        memory_bytes: overrides
            .memory_limit_mb
            .filter(|mb| *mb > 0)
            .and_then(|mb| mb.checked_mul(1024 * 1024)),
        pids_max: None,
        disk_bytes: None,
    }
}

/// The grants one spawn asks for: its workspace read-write plus the mounts the
/// executor honours (a container ignores read-write mounts, as it always has).
fn grants(policy: &SandboxPolicy) -> Option<Vec<PathGrant>> {
    let mut grants = facts::grant_ceiling(policy)?;
    if policy.backend == SandboxBackendKind::Docker {
        let writable: Vec<_> = policy
            .read_write_mounts
            .iter()
            .filter_map(|path| path.to_str())
            .collect();
        grants.retain(|grant| {
            Path::new(&grant.path) == policy.workspace_root
                || !(grant.write && writable.contains(&grant.path.as_str()))
        });
    }
    Some(grants)
}

fn ensure_selected(plan: &BoxPlan, policy: &SandboxPolicy) -> Result<(), PlanError> {
    let expected = match policy.backend {
        SandboxBackendKind::Local => PlannedExecutor::NativeJail,
        SandboxBackendKind::Docker => PlannedExecutor::Docker,
        SandboxBackendKind::None => return Err(unavailable("the policy selects no sandbox")),
    };
    if plan.executor != expected {
        return Err(unavailable("the plan selected another executor"));
    }
    Ok(())
}

/// The TinyBox facts of the executor `policy` selects.
async fn probe(
    config: &Config,
    policy: &SandboxPolicy,
) -> Result<tinysecurity_bus::SandboxExecutor2, PlanError> {
    match policy.backend {
        SandboxBackendKind::Local => {
            let status = crate::modules::tinybox::jail_status(config)
                .await
                .map_err(|error| unavailable(format!("jail status: {error}")))?;
            Ok(executors::jail(&status))
        }
        SandboxBackendKind::Docker => {
            let available = super::docker::is_docker_available(config)
                .await
                .map_err(|error| unavailable(format!("docker status: {error}")))?;
            Ok(executors::docker(available))
        }
        SandboxBackendKind::None => Err(unavailable("the policy selects no sandbox")),
    }
}

/// Plan one spawn through the module.
///
/// The module plans the host's own network only after an exact human approval, which
/// no sandboxed spawn has a surface to ask for yet. A spawn whose policy allows the
/// network is therefore planned as asked first; if the module answers that it needs
/// that approval, the spawn is planned again with no network. The module only ever
/// narrows what the host asked for: the command never gets more than the policy
/// allowed, and one that needs the network fails inside the sandbox instead of
/// running with it.
pub(crate) async fn plan(request: SpawnRequest<'_>) -> Result<PlannedSpawn, PlanError> {
    let asked = facts::network_ceiling(request.policy);
    match plan_with(&request, asked.clone()).await {
        Err(PlanError::Refused(SandboxDenial2::Approval)) if asked == NetworkPolicy::Host => {
            tracing::warn!(
                "[sandbox:plan] host network needs a human approval; planning without network"
            );
            plan_with(&request, NetworkPolicy::None).await
        }
        other => other,
    }
}

async fn plan_with(
    request: &SpawnRequest<'_>,
    network: NetworkPolicy,
) -> Result<PlannedSpawn, PlanError> {
    let SpawnRequest {
        config,
        policy,
        source,
        operation,
        host_off,
        module,
    } = *request;
    let saas = crate::core::runtime::is_saas();
    let tenant = crate::core::runtime::tenant::current_tenant()
        .map_err(|_| unavailable("no tenant scope"))?;
    let subject = crate::modules::security::policy_translation::subject_scope(
        &tenant,
        &config.workspace_dir,
        saas,
    )
    .map_err(unavailable)?;
    let proxy = module.proxy(config).await?;
    let executor = match module {
        ModuleSource::Production => probe(config, policy).await?,
        ModuleSource::Fixed(fixed) => fixed.executor.clone(),
    };
    let scope = facts::scope(&facts::FactsInput {
        config,
        policy,
        source,
        subject: subject.clone(),
        saas,
        host_off,
        executors: vec![executor],
    })
    .ok_or_else(|| unavailable("a sandbox path is not valid text"))?;
    let stream = AuditStream::open(config, subject.clone())
        .await
        .map_err(unavailable)?;
    registry::ensure(&proxy, &scope, stream.stream_id()).await?;

    let operation_json = operation
        .canonical()
        .ok_or_else(|| unavailable("the working directory is not valid text"))?;
    let grants = grants(policy).ok_or_else(|| unavailable("a sandbox path is not valid text"))?;
    let ordered = stream.lock().await;
    let generation = client::generation(&proxy).await.map_err(unavailable)?;
    let audit = stream.correlation(generation).await.map_err(unavailable)?;
    let call_id = audit.event_id.clone();
    let planned = SandboxPlanRequest2 {
        host_approval: None,
        scope_id: scope.scope_id.clone(),
        subject: subject.clone(),
        source_id: source.source_id().into(),
        provenance: source.provenance().into(),
        call_id,
        operation_json,
        grants,
        network,
        limits: limits(policy),
        deny_subprocesses: false,
        accept_process_isolation: source.accepts_process_isolation(),
        generation,
        ttl_ms: facts::PLAN_TTL_MS,
        audit,
    };
    let response = client::plan(&proxy, planned.clone()).await;
    drop(ordered);
    let response = match response {
        Ok(response) => response,
        Err(error) => {
            registry::forget(&scope.scope_id).await;
            return Err(unavailable(error));
        }
    };
    if let Some(denial) = response.denial {
        if denial == SandboxDenial2::InvalidContext {
            // The module no longer holds the scope (a reload): register it again next time.
            registry::forget(&scope.scope_id).await;
        }
        tracing::warn!(?denial, "[sandbox:plan] the module refused the spawn");
        return Err(PlanError::Refused(denial));
    }
    let artifact = match (response.decision.verdict, response.artifact) {
        (Verdict::Allow, Some(artifact)) => artifact,
        _ => return Err(unavailable("the module returned no plan")),
    };
    if response.audit.is_none_or(|audit| !audit.committed) {
        return Err(unavailable("the plan was not audited"));
    }
    if artifact.scope_id != scope.scope_id || artifact.subject != subject {
        return Err(unavailable("the plan names another scope"));
    }
    let home = super::local_grants::grant_home(saas, dirs::home_dir);
    let box_plan = BoxPlan::from_artifact(&artifact, home.as_deref())
        .map_err(|error: PlanMappingError| unavailable(error.to_string()))?;
    ensure_selected(&box_plan, policy)?;
    tracing::debug!(
        executor = ?box_plan.executor,
        grants = artifact.grants.len(),
        "[sandbox:plan] the module planned the spawn"
    );
    Ok(PlannedSpawn {
        plan: box_plan,
        artifact,
        request: planned,
        subject,
        stream,
        proxy,
    })
}

impl PlannedSpawn {
    /// The policy the spawn runs under: the caller's policy narrowed to what the module
    /// planned. Mounts, network reach and container bounds come from the artifact, never
    /// from the policy that was asked.
    pub(crate) fn apply(&self, policy: &SandboxPolicy) -> SandboxPolicy {
        let mut applied = policy.clone();
        let workspace = policy.workspace_root.to_str();
        let paths = |list: &[String]| -> Vec<std::path::PathBuf> {
            list.iter()
                .filter(|path| Some(path.as_str()) != workspace)
                .map(std::path::PathBuf::from)
                .collect()
        };
        applied.read_only_mounts = paths(&self.plan.jail.read_only);
        applied.read_write_mounts = paths(&self.plan.jail.read_write);
        applied.allow_network = self.plan.jail.allow_net;
        if let Some(overrides) = applied.docker_overrides.as_mut() {
            if !applied.allow_network {
                // The container's own network setting must follow the plan too, or a
                // narrowed plan would still start the container on a network.
                overrides.network = Some("none".to_owned());
            }
            let limits = &self.artifact.limits;
            overrides.memory_limit_mb =
                limits.memory_bytes.map(|bytes| bytes.div_ceil(1024 * 1024));
            overrides.cpu_limit = limits
                .cpu_millicores
                .map(|millis| f64::from(millis) / 1000.0);
        }
        applied
    }
}

/// Plan `request`, reserve the plan and return the policy the spawn must run under.
///
/// This is the one call a spawn site makes. The plan is reserved here, immediately
/// before the caller launches it, so it cannot be admitted twice and the audit trail
/// shows plan and admission for the same operation.
pub(crate) async fn admit(request: SpawnRequest<'_>) -> Result<SandboxPolicy, PlanError> {
    let planned = plan(request).await?;
    consume(&planned).await?;
    Ok(planned.apply(request.policy))
}

/// Reserve `planned` once, immediately before launching it.
pub(crate) async fn consume(planned: &PlannedSpawn) -> Result<(), PlanError> {
    let _ordered = planned.stream.lock().await;
    let generation = client::generation(&planned.proxy)
        .await
        .map_err(unavailable)?;
    let audit = planned
        .stream
        .correlation(generation)
        .await
        .map_err(unavailable)?;
    let response = client::consume(
        &planned.proxy,
        ConsumeSandboxPlanRequest2 {
            request: planned.request.clone(),
            artifact: planned.artifact.clone(),
            subject: planned.subject.clone(),
            audit,
        },
    )
    .await
    .map_err(unavailable)?;
    if response.permitted {
        return Ok(());
    }
    let denial = response.denial.unwrap_or(SandboxDenial2::InvalidContext);
    tracing::warn!(
        ?denial,
        "[sandbox:plan] the module refused to admit the spawn"
    );
    Err(PlanError::Refused(denial))
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "integration_tests.rs"]
mod integration_tests;
