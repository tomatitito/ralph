use regex::Regex;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::state::SharedState;
use crate::token_counter::TokenCounter;

/// Commands that can be sent from the monitor to the controller
#[derive(Debug, Clone)]
pub enum ProcessCommand {
    /// Kill the process due to context limit
    Kill,
}

/// Output monitor that reads from stdout/stderr and updates shared state
pub struct OutputMonitor {
    config: Arc<Config>,
    state: Arc<SharedState>,
    token_counter: TokenCounter,
    promise_regex: Regex,
    cmd_tx: mpsc::Sender<ProcessCommand>,
    warning_emitted: bool,
}

impl OutputMonitor {
    /// Create a new OutputMonitor
    pub fn new(
        config: Arc<Config>,
        state: Arc<SharedState>,
        cmd_tx: mpsc::Sender<ProcessCommand>,
    ) -> Self {
        let token_counter = TokenCounter::new(config.context_limit.estimation_method);
        // Match <promise>TEXT</promise> pattern
        let promise_regex = Regex::new(&format!(
            r"<promise>{}</promise>",
            regex::escape(&config.completion_promise)
        ))
        .expect("Invalid promise regex");

        Self {
            config,
            state,
            token_counter,
            promise_regex,
            cmd_tx,
            warning_emitted: false,
        }
    }

    /// Monitor a stream (stdout or stderr) for output
    pub async fn monitor_stream<R>(
        &mut self,
        reader: &mut BufReader<R>,
        stream_name: &str,
    ) -> crate::error::Result<()>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("{} stream closed", stream_name);
                    break;
                }
                Ok(_) => {
                    self.process_line(&line).await?;
                }
                Err(e) => {
                    warn!("{} read error: {}", stream_name, e);
                    break;
                }
            }
        }
        Ok(())
    }

    /// Process a single line of output
    async fn process_line(&mut self, line: &str) -> crate::error::Result<()> {
        // Update output buffer
        self.state.append_output(line).await;

        // Update token count
        let tokens = self.token_counter.count(line);
        self.state.add_tokens(tokens).await;
        let total_tokens = self.state.get_token_count().await;

        // Check for warning threshold
        if !self.warning_emitted && total_tokens >= self.config.context_limit.warning_threshold {
            warn!(
                "Context limit warning: {} tokens (threshold: {})",
                total_tokens, self.config.context_limit.warning_threshold
            );
            self.warning_emitted = true;
        }

        // Check for context limit
        if total_tokens >= self.config.context_limit.max_tokens {
            info!(
                "Context limit reached: {} tokens (limit: {})",
                total_tokens, self.config.context_limit.max_tokens
            );
            let _ = self.cmd_tx.send(ProcessCommand::Kill).await;
            return Ok(());
        }

        // Check for promise
        if self.promise_regex.is_match(line) {
            info!(
                "Promise found in output: {}",
                self.config.completion_promise
            );
            self.state
                .set_promise_found(self.config.completion_promise.clone())
                .await;
        }

        Ok(())
    }
}

/// Spawn monitor tasks for stdout and stderr
pub fn spawn_monitors(
    config: Arc<Config>,
    state: Arc<SharedState>,
    mut stdout: BufReader<tokio::process::ChildStdout>,
    mut stderr: BufReader<tokio::process::ChildStderr>,
    cmd_tx: mpsc::Sender<ProcessCommand>,
) -> (tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>) {
    let config_stdout = Arc::clone(&config);
    let state_stdout = Arc::clone(&state);
    let cmd_tx_stdout = cmd_tx.clone();

    let stdout_handle = tokio::spawn(async move {
        let mut monitor = OutputMonitor::new(config_stdout, state_stdout, cmd_tx_stdout);
        if let Err(e) = monitor.monitor_stream(&mut stdout, "stdout").await {
            warn!("stdout monitor error: {}", e);
        }
    });

    let config_stderr = Arc::clone(&config);
    let state_stderr = Arc::clone(&state);
    let cmd_tx_stderr = cmd_tx;

    let stderr_handle = tokio::spawn(async move {
        let mut monitor = OutputMonitor::new(config_stderr, state_stderr, cmd_tx_stderr);
        if let Err(e) = monitor.monitor_stream(&mut stderr, "stderr").await {
            warn!("stderr monitor error: {}", e);
        }
    });

    (stdout_handle, stderr_handle)
}
