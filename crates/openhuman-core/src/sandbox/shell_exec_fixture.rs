//! Test-only stand-in for the module's native shell: unit tests do not load a
//! native TinyBox artifact, so unjailed runs execute here. The module's own
//! behaviour (shell selection, environment, scratch) is covered by its crate and
//! by the released-artifact test; this fixture only has to run a command, and
//! like the module it keeps what a command printed before its deadline stopped
//! it.

use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

use super::{NativeShellRun, SandboxExecResult};

/// Variables the module's safe environment snapshot carries.
const SNAPSHOT: &[&str] = &[
    "PATH", "HOME", "TERM", "LANG", "LC_ALL", "LC_CTYPE", "USER", "SHELL", "TMPDIR",
];

pub(crate) async fn run(run: NativeShellRun) -> anyhow::Result<SandboxExecResult> {
    let bash = ["/usr/bin/bash", "/bin/bash"]
        .into_iter()
        .find(|path| std::path::Path::new(path).exists());
    let mut command = tokio::process::Command::new(if cfg!(windows) {
        "cmd"
    } else {
        bash.unwrap_or("sh")
    });
    if cfg!(windows) {
        command.arg("/C").arg(&run.command);
    } else if bash.is_some() && !run.preserve_pipeline_status {
        command
            .arg("-c")
            .arg(format!("set -o pipefail\n{}", run.command));
    } else {
        command.arg("-c").arg(&run.command);
    }
    command.env_clear();
    for name in SNAPSHOT {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    if cfg!(windows) {
        for name in ["SystemRoot", "WINDIR", "COMSPEC", "PATHEXT", "TEMP", "TMP"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
    }
    if run.python_utf8 {
        // The module defaults these unless the parent environment sets them.
        for (name, default) in [("PYTHONUTF8", "1"), ("PYTHONIOENCODING", "utf-8")] {
            command.env(
                name,
                std::env::var_os(name).unwrap_or_else(|| default.into()),
            );
        }
    }
    command.envs(&run.env);
    command
        .current_dir(&run.cwd)
        .stdin(if run.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    tinytools_std::timeout::own_process_group(command.as_std_mut());
    let mut child = command
        .spawn()
        .map_err(|error| anyhow::anyhow!("Failed to execute command: {error}"))?;
    let pid = child.id();
    let input = run.stdin;
    let mut stdin_pipe = child.stdin.take();
    // Read both streams as the command runs, like the module does, so a
    // command stopped at its deadline keeps what it printed before.
    let (mut stdout_pipe, mut stderr_pipe) = (child.stdout.take(), child.stderr.take());
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let finished = tokio::time::timeout(run.timeout, async {
        let send_input = async {
            if let (Some(input), Some(mut stdin)) = (input, stdin_pipe.take()) {
                let _ = stdin.write_all(&input).await;
            }
        };
        let (status, (), (), ()) = tokio::join!(
            child.wait(),
            drain(&mut stdout_pipe, &mut stdout),
            drain(&mut stderr_pipe, &mut stderr),
            send_input,
        );
        status
    })
    .await;
    let (exit_code, timed_out) = match finished {
        Ok(status) => (status?.code().unwrap_or(-1), false),
        Err(_) => {
            // Stop the command and everything it started, then give the
            // readers a moment to drain what is left in the pipes.
            if let Some(pid) = pid {
                tinytools_std::timeout::kill_process_group(pid);
            }
            let _ = child.start_kill();
            let _ = tokio::time::timeout(Duration::from_secs(2), async {
                tokio::join!(
                    drain(&mut stdout_pipe, &mut stdout),
                    drain(&mut stderr_pipe, &mut stderr),
                )
            })
            .await;
            (-1, true)
        }
    };
    // The module streams output into the caller's meter as it reads it; the
    // fixture folds the collected output in the same way.
    if let Some(meter) = &run.meter {
        meter.record(&stdout);
        meter.record(&stderr);
    }
    Ok(SandboxExecResult {
        exit_code,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
    })
}

/// Appends what `pipe` delivers to `into` until it closes. Reads are
/// cancel-safe, so a drain cut short by the deadline keeps every chunk it read,
/// and a later call picks up where it stopped.
async fn drain(pipe: &mut Option<impl AsyncRead + Unpin>, into: &mut Vec<u8>) {
    let Some(pipe) = pipe else {
        return;
    };
    let mut chunk = [0u8; 8192];
    while let Ok(read) = pipe.read(&mut chunk).await {
        if read == 0 {
            break;
        }
        into.extend_from_slice(&chunk[..read]);
    }
}
