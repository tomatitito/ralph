use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::error::Result;
use crate::json_events::TokenUsage;
use crate::monitor::{spawn_monitors, MonitorResult, ProcessCommand};
use crate::process::ClaudeProcess;
use crate::state::SharedState;

/// The reason a Claude agent invocation ended
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    /// Process exited naturally
    Natural,
    /// Process was killed due to context limit
    ContextLimit,
    /// Process was killed due to shutdown signal
    Shutdown,
}

/// Result of a single agent invocation
#[derive(Debug, Clone)]
pub struct AgentResult {
    /// The output from the agent
    pub output: String,
    /// The promise text if found, None otherwise
    pub promise_found: Option<String>,
    /// Estimated token count of the output
    pub token_count: usize,
    /// Why the agent invocation ended
    pub exit_reason: ExitReason,
    /// Session ID captured from Claude Code
    pub session_id: Option<String>,
    /// Detailed token usage from Claude Code
    pub token_usage: Option<TokenUsage>,
}

impl AgentResult {
    /// Create an AgentResult with a promise found
    pub fn with_promise(promise: &str) -> Self {
        Self {
            output: String::new(),
            promise_found: Some(promise.to_string()),
            token_count: 0,
            exit_reason: ExitReason::Natural,
            session_id: None,
            token_usage: None,
        }
    }

    /// Create an AgentResult without a promise
    pub fn without_promise() -> Self {
        Self {
            output: String::new(),
            promise_found: None,
            token_count: 0,
            exit_reason: ExitReason::Natural,
            session_id: None,
            token_usage: None,
        }
    }

    /// Check if the completion promise was fulfilled
    pub fn is_fulfilled(&self) -> bool {
        self.promise_found.is_some()
    }

    /// Apply monitor result to this agent result
    pub fn with_monitor_result(mut self, monitor_result: MonitorResult) -> Self {
        self.session_id = monitor_result.session_id;
        self.token_usage = monitor_result.token_usage;
        self
    }
}

/// Trait for agent implementations (real Claude or mock)
#[async_trait]
pub trait Agent: Send + Sync {
    /// Run the agent with the given prompt
    async fn run(&self, prompt: &str) -> Result<AgentResult>;
}

/// Production implementation of Agent that spawns Claude subprocess
pub struct ClaudeAgent {
    config: Arc<Config>,
}

impl ClaudeAgent {
    /// Create a new ClaudeAgent with the given configuration
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Agent for ClaudeAgent {
    async fn run(&self, prompt: &str) -> Result<AgentResult> {
        let state = SharedState::new_shared();

        // Create command channel for monitors to send kill commands
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<ProcessCommand>(1);

        // Spawn Claude process with stdin (for headless mode)
        debug!(
            "Spawning Claude process: {} {:?}",
            self.config.claude_path, self.config.claude_args
        );
        let mut process =
            ClaudeProcess::spawn_with_stdin(&self.config.claude_path, &self.config.claude_args, prompt)
                .await?;

        // Take stdout and stderr for monitoring
        let stdout = process.stdout.take().expect("stdout not available");
        let stderr = process.stderr.take().expect("stderr not available");

        // Spawn monitor tasks
        let (stdout_handle, stderr_handle) = spawn_monitors(
            Arc::clone(&self.config),
            Arc::clone(&state),
            stdout,
            stderr,
            cmd_tx,
        );

        // Wait for process to exit or kill command
        let exit_reason = tokio::select! {
            // Wait for process to exit naturally
            status = process.wait() => {
                match status {
                    Ok(s) => {
                        info!("Claude process exited with status: {:?}", s);
                        ExitReason::Natural
                    }
                    Err(e) => {
                        warn!("Error waiting for Claude process: {}", e);
                        ExitReason::Natural
                    }
                }
            }
            // Or receive kill command from monitor (context limit)
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    ProcessCommand::Kill => {
                        info!("Killing Claude process due to context limit");
                        let _ = process.kill().await;
                        ExitReason::ContextLimit
                    }
                }
            }
        };

        // Wait for monitors to finish and get results
        let (stdout_result, _) = tokio::join!(stdout_handle, stderr_handle);
        let monitor_result = stdout_result.unwrap_or_default();

        // Build result
        let output = state.get_output().await;
        let token_count = state.get_token_count().await;
        let promise_found = state.get_promise_text().await;

        Ok(AgentResult {
            output,
            promise_found,
            token_count,
            exit_reason,
            session_id: monitor_result.session_id,
            token_usage: monitor_result.token_usage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_result_with_promise() {
        let result = AgentResult::with_promise("TASK COMPLETE");
        assert!(result.is_fulfilled());
        assert_eq!(result.promise_found, Some("TASK COMPLETE".to_string()));
    }

    #[test]
    fn test_agent_result_without_promise() {
        let result = AgentResult::without_promise();
        assert!(!result.is_fulfilled());
        assert_eq!(result.promise_found, None);
    }
}
