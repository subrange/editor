use anyhow::{Context, Result};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Result of running a command
#[derive(Debug)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

/// Run a shell command with timeout
pub async fn run_command(
    cmd: &str,
    timeout_secs: u64,
) -> Result<CommandResult> {
    let child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context(format!("Failed to spawn command: {}", cmd))?;

    let timeout_duration = Duration::from_secs(timeout_secs);

    // Try to wait for the process with timeout
    let result = timeout(timeout_duration, child.wait_with_output()).await;

    match result {
        Ok(Ok(output)) => {
            Ok(CommandResult {
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                timed_out: false,
            })
        }
        Ok(Err(e)) => Err(e.into()),
        Err(_) => {
            // Timeout occurred
            Ok(CommandResult {
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Timeout after {}s", timeout_secs),
                timed_out: true,
            })
        }
    }
}

/// Run a command synchronously (wrapper for async version)
pub fn run_command_sync(cmd: &str, timeout_secs: u64) -> Result<CommandResult> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(run_command(cmd, timeout_secs))
}

/// Check if a command/binary exists
pub fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Clean up a file if it exists
pub fn cleanup_file(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path)
            .context(format!("Failed to remove file: {}", path.display()))?;
    }
    Ok(())
}

/// Clean up multiple files matching a pattern
pub fn cleanup_pattern(pattern: &str) -> Result<usize> {
    let mut count = 0;
    for entry in glob::glob(pattern)? {
        if let Ok(path) = entry {
            cleanup_file(&path)?;
            count += 1;
        }
    }
    Ok(count)
}