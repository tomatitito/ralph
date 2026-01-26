use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdout, Command};

use crate::error::{RalphError, Result};

/// Wrapper around a Claude subprocess
pub struct ClaudeProcess {
    child: Child,
    pub stdout: Option<BufReader<ChildStdout>>,
    pub stderr: Option<BufReader<ChildStderr>>,
}

impl ClaudeProcess {
    /// Spawn a new Claude process
    pub async fn spawn(claude_path: &str, args: &[String], prompt: &str) -> Result<Self> {
        let mut cmd = Command::new(claude_path);
        cmd.args(args)
            .arg("-p")
            .arg(prompt)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(RalphError::ProcessSpawnError)?;

        let stdout = child.stdout.take().map(BufReader::new);
        let stderr = child.stderr.take().map(BufReader::new);

        Ok(Self {
            child,
            stdout,
            stderr,
        })
    }

    /// Spawn a new Claude process with prompt via stdin (for --print mode)
    pub async fn spawn_with_stdin(
        claude_path: &str,
        args: &[String],
        prompt: &str,
    ) -> Result<Self> {
        let mut cmd = Command::new(claude_path);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(RalphError::ProcessSpawnError)?;

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .await
                .map_err(RalphError::ProcessIoError)?;
            stdin.flush().await.map_err(RalphError::ProcessIoError)?;
            // Drop stdin to close it and signal EOF
        }

        let stdout = child.stdout.take().map(BufReader::new);
        let stderr = child.stderr.take().map(BufReader::new);

        Ok(Self {
            child,
            stdout,
            stderr,
        })
    }

    /// Wait for the process to exit and return the exit status
    pub async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        self.child.wait().await.map_err(RalphError::ProcessIoError)
    }

    /// Kill the process
    pub async fn kill(&mut self) -> Result<()> {
        self.child.kill().await.map_err(RalphError::ProcessIoError)
    }

    /// Check if the process has exited
    pub fn try_wait(&mut self) -> Result<Option<std::process::ExitStatus>> {
        self.child.try_wait().map_err(RalphError::ProcessIoError)
    }

    /// Get the process ID
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }
}

/// Read lines from a buffered reader
pub async fn read_lines(
    reader: &mut BufReader<impl tokio::io::AsyncRead + Unpin>,
) -> Result<Option<String>> {
    let mut line = String::new();
    match reader.read_line(&mut line).await {
        Ok(0) => Ok(None), // EOF
        Ok(_) => Ok(Some(line)),
        Err(e) => Err(RalphError::ProcessIoError(e)),
    }
}
