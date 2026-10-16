//! The one gate for shell spawns whose argv the host did not author (issue
//! #7328, phase 5).
//!
//! A scheduled shell job is text a model, a flow or an operator typed into a
//! schedule long before it runs. It reaches the same isolation floor as the
//! agent's shell tool: the TinyBox native jail (or the configured Docker
//! container), refused when the jail is missing, inactive (Noop) or not
//! suitable for untrusted code. The only way around the jail is the operator's
//! own opt-out (`OPENHUMAN_SANDBOX=off`, `[sandbox] enabled = false` or
//! `[components] sandbox = false`), and SaaS never honours it: SaaS commands
//! run in the user's container or not at all.
//!
//! `scripts/ci/spawner-inventory.json` names every caller of this gate.

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use super::types::{SandboxBackendKind, SandboxExecRequest, SandboxExecResult};
use crate::config::Config;

/// One untrusted shell command, already authorized by the security module.
pub(crate) struct UntrustedShell<'a> {
    /// Loaded configuration of the scope the command belongs to.
    pub(crate) config: &'a Config,
    /// Shell text.
    pub(crate) command: &'a str,
    /// Working directory; also the jail's writable workspace.
    pub(crate) cwd: &'a Path,
    /// The core's private state directory (per-call scratch is created below).
    pub(crate) state_dir: &'a Path,
    /// Wall-clock budget.
    pub(crate) timeout: Duration,
    /// Keep the shell's own last-stage pipeline status instead of `pipefail`.
    pub(crate) preserve_pipeline_status: bool,
    /// Extra environment variables for the command (none for a scheduled job).
    pub(crate) env: &'a BTreeMap<String, String>,
    /// Bytes written to the command's stdin (a hook's event JSON), if any.
    pub(crate) stdin: Option<&'a [u8]>,
}

/// Largest stdin the Docker path can carry. The container runs `sh -c <text>`
/// with no stdin channel, so input rides inside the command as base64 and must
/// fit one argument (Linux caps a single argument at 128 KiB).
const DOCKER_STDIN_MAX: usize = 72 * 1024;

/// Wrap `command` so it reads `stdin` as its standard input.
fn with_inline_stdin(command: &str, stdin: &[u8]) -> anyhow::Result<String> {
    use base64::Engine as _;
    if stdin.len() > DOCKER_STDIN_MAX {
        anyhow::bail!(
            "stdin of {} bytes exceeds the {DOCKER_STDIN_MAX}-byte limit of the Docker sandbox",
            stdin.len()
        );
    }
    let encoded = base64::engine::general_purpose::STANDARD.encode(stdin);
    Ok(format!(
        "printf '%s' '{encoded}' | base64 -d | (\n{command}\n)"
    ))
}

/// Run `spawn` under the isolation floor.
///
/// Errors are refusals, never a fallback: a jail below the floor yields
/// [`super::ops::IsolationFloor`], a policy that resolves to no sandbox at all
/// is an error, and so is a Docker backend that is unavailable.
pub(crate) async fn run(spawn: UntrustedShell<'_>) -> anyhow::Result<SandboxExecResult> {
    let UntrustedShell {
        config,
        command,
        cwd,
        state_dir,
        timeout,
        preserve_pipeline_status,
        env,
        stdin,
    } = spawn;
    let saas = crate::core::runtime::is_saas();
    // The scheduler has no dispatch context of its own; the task's ambient one
    // answers the component question (#7801 transitional).
    let ctx = crate::core::runtime::DispatchCtx::ambient();
    if !saas
        && (super::ops::sandbox_disabled_by_host(&ctx)
            || super::ops::config_disables_sandbox(config))
    {
        // The operator turned isolation off for this process. This is the only
        // unjailed path, it is never reachable under SaaS, and it is logged.
        tracing::warn!("[sandbox:untrusted] operator opt-out: running an untrusted shell unjailed");
        crate::config::ensure_usable_cwd(cwd)?;
        std::fs::create_dir_all(state_dir)
            .map_err(|error| anyhow::anyhow!("Cannot prepare sandbox state directory: {error}"))?;
        let mut shell = super::shell_exec::NativeShellRun::new(command, cwd, state_dir, timeout);
        shell.env = env.clone();
        shell.preserve_pipeline_status = preserve_pipeline_status;
        shell.stdin = stdin.map(<[u8]>::to_vec);
        return super::shell_exec::run(config, shell).await;
    }

    let mut policy = super::ops::resolve_command_policy_with_config(&ctx, config, cwd, state_dir)
        .await
        .map_err(|error| anyhow::anyhow!(error))?;
    // Landlock named explicitly in the config must really be the backend.
    let explicitly_requested = !saas
        && matches!(
            config.sandbox.backend,
            crate::config::SandboxBackend::Landlock
        )
        && policy.backend == SandboxBackendKind::Local;
    match policy.backend {
        SandboxBackendKind::Local => {
            crate::config::ensure_usable_cwd(cwd)?;
            policy = super::local_grants::with_resolved_grants(config, &policy).await?;
            tracing::debug!("[sandbox:untrusted] running under the native jail");
            let extra_env: HashMap<OsString, OsString> = env
                .iter()
                .map(|(name, value)| (name.into(), value.into()))
                .collect();
            super::native_jail::execute_with_stdin(
                config,
                &policy,
                command,
                cwd,
                &extra_env,
                timeout,
                explicitly_requested,
                preserve_pipeline_status,
                None,
                stdin.map(<[u8]>::to_vec),
            )
            .await
        }
        SandboxBackendKind::Docker => {
            crate::config::ensure_usable_cwd(&policy.workspace_root)?;
            tracing::debug!("[sandbox:untrusted] running in the sandbox container");
            let command = match stdin {
                Some(stdin) => with_inline_stdin(command, stdin)?,
                None => command.to_owned(),
            };
            let request = SandboxExecRequest {
                command,
                working_dir: cwd.to_path_buf(),
                env: env
                    .iter()
                    .map(|(name, value)| (name.into(), value.into()))
                    .collect(),
                timeout,
            };
            super::docker::docker_exec(config, &policy, &request).await
        }
        SandboxBackendKind::None => {
            anyhow::bail!("an untrusted spawn resolved to no sandbox backend")
        }
    }
}

#[cfg(test)]
#[path = "untrusted_tests.rs"]
mod tests;
