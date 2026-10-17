//! Host adapter from a TinySecurity `SandboxArtifact2` (the declarative sandbox
//! plan) onto the TinyBox vocabulary the host drives (issue #7328, phase 5).
//!
//! The security module decides *what may run*: which roots, which network
//! reach, which bounds, which minimum isolation. This adapter only translates
//! that decision into TinyBox requests ([`NativeJailConfig`], [`NetworkPolicy`],
//! [`ResourceLimits`]) and refuses anything TinyBox cannot enforce exactly. It
//! never widens a plan and never falls back: a `Noop` plan that is not the module's
//! own `host_run` plan, an egress proxy the host does not run, and a CPU-time bound
//! (TinyBox has only CPU throughput) are errors. Live availability and capability checks stay at the launch boundary.
//!
//! The credential-store floor (`deny_read`) is not part of the plan; the host
//! adds it for jails whose reads are allow-by-default
//! ([`super::local_grants::credential_store_paths`]).

use std::path::Path;

use tinysecurity_bus::{
    IsolationLevel2, NetworkPolicy as PlannedNetwork, SandboxArtifact2, SandboxBackend,
    SandboxConstraint2,
};

use crate::modules::tinybox::contract::{
    JailEnforcement, NativeJailConfig, NetworkPolicy, ResourceLimits,
};

/// The TinyBox executor a plan selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannedExecutor {
    /// The native path jail (Landlock, AppContainer; Seatbelt only for a plan whose
    /// request accepted process isolation).
    NativeJail,
    /// A one-shot container.
    Docker,
    /// No jail: the operator disabled the sandbox and the module planned a run on the
    /// host. Only a `host_run` artifact maps here.
    Host,
}

/// A plan translated for TinyBox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxPlan {
    /// Executor to launch under.
    pub executor: PlannedExecutor,
    /// Grants, subprocess rule and isolation requirement for the native jail.
    pub jail: NativeJailConfig,
    /// Network policy for the box.
    pub network: NetworkPolicy,
    /// Resource bounds for the box.
    pub resources: ResourceLimits,
    /// Whether the executor mounts the workspace. `false` for a plan the module made
    /// with no workspace grant (a container with nothing mounted).
    pub workspace: bool,
}

/// Why a plan cannot be mapped onto what TinyBox enforces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanMappingError {
    /// The plan selected the no-op backend, which confines nothing.
    NoopBackend,
    /// The plan selected an executor the host does not drive yet.
    ExecutorNotWired,
    /// Mediated egress needs an exclusive validated proxy the host does not run.
    EgressProxyUnavailable,
    /// A CPU-time bound; TinyBox enforces CPU throughput only.
    CpuTimeUnenforceable,
    /// A zero bound is never a limit.
    ZeroLimit,
    /// A grant path is not valid UTF-8 text for the bus.
    InvalidGrant,
    /// A plan marked as a run on the host that also claims a jail, grants or constraints.
    MalformedHostRun,
}

impl std::fmt::Display for PlanMappingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoopBackend => "the sandbox plan selected the no-op backend",
            Self::ExecutorNotWired => {
                "the sandbox plan selected an executor the host does not drive"
            }
            Self::EgressProxyUnavailable => "the sandbox plan needs a mediated egress proxy",
            Self::CpuTimeUnenforceable => {
                "the sandbox plan bounds CPU time, which TinyBox cannot enforce"
            }
            Self::ZeroLimit => "the sandbox plan carries a zero resource bound",
            Self::InvalidGrant => "the sandbox plan carries a grant that is not valid text",
            Self::MalformedHostRun => "the sandbox plan for a run on the host claims a jail",
        })
    }
}

impl std::error::Error for PlanMappingError {}

impl BoxPlan {
    /// Translate `artifact`. `home` is the operator home whose credential stores
    /// the jail denies reading (`None` for a tenant).
    pub fn from_artifact(
        artifact: &SandboxArtifact2,
        home: Option<&Path>,
    ) -> Result<Self, PlanMappingError> {
        if artifact.host_run {
            return Self::host_run(artifact);
        }
        let executor = match &artifact.backend {
            SandboxBackend::Jail => PlannedExecutor::NativeJail,
            SandboxBackend::Docker => PlannedExecutor::Docker,
            SandboxBackend::Noop => return Err(PlanMappingError::NoopBackend),
            SandboxBackend::Namespace | SandboxBackend::MicroVm => {
                return Err(PlanMappingError::ExecutorNotWired)
            }
        };
        let network = match artifact.network {
            PlannedNetwork::None => NetworkPolicy::Denied,
            PlannedNetwork::Host => NetworkPolicy::Open,
            PlannedNetwork::EgressProxy => return Err(PlanMappingError::EgressProxyUnavailable),
        };
        let resources = resources(artifact)?;

        let mut read_only = Vec::new();
        let mut read_write = Vec::new();
        for grant in &artifact.grants {
            if grant.path.is_empty() {
                return Err(PlanMappingError::InvalidGrant);
            }
            if grant.write {
                read_write.push(grant.path.clone());
            } else {
                read_only.push(grant.path.clone());
            }
        }
        let isolated = matches!(
            artifact.minimum_isolation,
            IsolationLevel2::Kernel | IsolationLevel2::MicroVm
        );
        let jail = NativeJailConfig {
            read_only,
            read_write,
            allow_net: !matches!(network, NetworkPolicy::Denied),
            allow_subprocess: !artifact
                .constraints
                .contains(&SandboxConstraint2::Subprocesses),
            // A kernel plan only exists for constraints the executor enforces
            // exactly. A process-level plan (macOS Seatbelt, for an ordinary tool
            // command the request accepted it for) carries best-effort constraints,
            // which TinyBox applies as declared; it still refuses what it cannot
            // apply at all.
            enforcement: if isolated {
                JailEnforcement::Enforced
            } else {
                JailEnforcement::Declared
            },
            require_process_isolation: isolated,
            deny_read: if isolated {
                Vec::new()
            } else {
                super::local_grants::credential_store_paths(home)
            },
        };
        Ok(Self {
            executor,
            jail,
            network,
            resources,
            workspace: !artifact.no_workspace,
        })
    }

    /// The module's plan for a spawn the operator disabled the sandbox for: it runs on
    /// the host, so there is no jail to configure and nothing to grant. The module only
    /// issues it for a trusted operator source outside SaaS; anything else that claims
    /// to be a host run is malformed and refused.
    fn host_run(artifact: &SandboxArtifact2) -> Result<Self, PlanMappingError> {
        if artifact.backend != SandboxBackend::Noop
            || artifact.minimum_isolation != IsolationLevel2::None
            || !artifact.grants.is_empty()
            || !artifact.constraints.is_empty()
            || artifact.no_workspace
        {
            return Err(PlanMappingError::MalformedHostRun);
        }
        Ok(Self {
            executor: PlannedExecutor::Host,
            jail: NativeJailConfig {
                read_only: Vec::new(),
                read_write: Vec::new(),
                allow_net: true,
                allow_subprocess: true,
                enforcement: JailEnforcement::Declared,
                require_process_isolation: false,
                deny_read: Vec::new(),
            },
            network: NetworkPolicy::Open,
            resources: resources(artifact)?,
            workspace: true,
        })
    }
}

fn resources(artifact: &SandboxArtifact2) -> Result<ResourceLimits, PlanMappingError> {
    let limits = &artifact.limits;
    if limits.cpu_time_ms.is_some() {
        return Err(PlanMappingError::CpuTimeUnenforceable);
    }
    if limits.cpu_millicores == Some(0)
        || limits.memory_bytes == Some(0)
        || limits.pids_max == Some(0)
        || limits.disk_bytes == Some(0)
    {
        return Err(PlanMappingError::ZeroLimit);
    }
    // An absent bound keeps TinyBox's own default ceiling, never "unlimited".
    let defaults = ResourceLimits::default();
    Ok(ResourceLimits {
        cpu_millis: limits.cpu_millicores.unwrap_or(defaults.cpu_millis),
        memory_bytes: limits.memory_bytes.unwrap_or(defaults.memory_bytes),
        pids_max: limits.pids_max.unwrap_or(defaults.pids_max),
        disk_bytes: limits.disk_bytes.unwrap_or(defaults.disk_bytes),
    })
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;
