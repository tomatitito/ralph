//! Output monitoring for Claude process streams.
//!
//! In headless mode, stdout produces JSON events while stderr is plain text.

use regex::Regex;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tracing::{debug, info, trace, warn};

use crate::config::Config;
use crate::json_events::{ClaudeEvent, TokenUsage};
use crate::state::SharedState;

/// Commands that can be sent from the monitor to the controller
#[derive(Debug, Clone)]
pub enum ProcessCommand {
    /// Kill the process due to context limit
    Kill,
}

/// Result from monitoring a Claude session
#[derive(Debug, Clone, Default)]
pub struct MonitorResult {
    /// Session ID captured from init or result event
    pub session_id: Option<String>,
    /// Token usage from the result event
    pub token_usage: Option<TokenUsage>,
}

/// JSON event monitor for stdout (in headless mode)
pub struct JsonEventMonitor {
    config: Arc<Config>,
    state: Arc<SharedState>,
    promise_regex: Regex,
    cmd_tx: mpsc::Sender<ProcessCommand>,
    warning_emitted: bool,
    /// Captured session ID
    session_id: Option<String>,
    /// Captured token usage
    token_usage: Option<TokenUsage>,
    /// Count of lines read
    line_count: u64,
    /// Count of events parsed successfully
    event_count: u64,
}

impl JsonEventMonitor {
    /// Create a new JsonEventMonitor
    pub fn new(
        config: Arc<Config>,
        state: Arc<SharedState>,
        cmd_tx: mpsc::Sender<ProcessCommand>,
    ) -> Self {
        // Match <promise>TEXT</promise> pattern
        let promise_regex = Regex::new(&format!(
            r"<promise>{}</promise>",
            regex::escape(&config.completion_promise)
        ))
        .expect("Invalid promise regex");

        Self {
            config,
            state,
            promise_regex,
            cmd_tx,
            warning_emitted: false,
            session_id: None,
            token_usage: None,
            line_count: 0,
            event_count: 0,
        }
    }

    /// Get the monitor result with captured session ID and token usage
    pub fn result(&self) -> MonitorResult {
        MonitorResult {
            session_id: self.session_id.clone(),
            token_usage: self.token_usage.clone(),
        }
    }

    /// Monitor stdout for JSON events
    pub async fn monitor_stream<R>(&mut self, reader: &mut BufReader<R>) -> crate::error::Result<()>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        info!("stdout monitor: starting to read JSON events");
        let mut line = String::new();
        loop {
            line.clear();
            trace!(
                "stdout monitor: waiting for next line (read {} lines so far)",
                self.line_count
            );
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    info!(
                        "stdout monitor: stream closed - read {} lines, parsed {} events",
                        self.line_count, self.event_count
                    );
                    break;
                }
                Ok(bytes) => {
                    self.line_count += 1;
                    trace!(
                        "stdout monitor: read line {} ({} bytes)",
                        self.line_count,
                        bytes
                    );
                    self.process_json_line(&line).await?;
                }
                Err(e) => {
                    warn!(
                        "stdout monitor: read error after {} lines: {}",
                        self.line_count, e
                    );
                    break;
                }
            }
        }
        Ok(())
    }

    /// Process a JSON event line
    async fn process_json_line(&mut self, line: &str) -> crate::error::Result<()> {
        let line = line.trim();
        if line.is_empty() {
            trace!("stdout monitor: skipping empty line");
            return Ok(());
        }

        // Store raw JSON for output
        self.state.append_output(line).await;
        self.state.append_output("\n").await;

        // Parse the JSON event
        let event = match ClaudeEvent::parse(line) {
            Ok(e) => e,
            Err(e) => {
                debug!(
                    "stdout monitor: failed to parse JSON event: {} - line: {}",
                    e,
                    if line.len() > 100 { &line[..100] } else { line }
                );
                return Ok(());
            }
        };

        self.event_count += 1;
        debug!(
            "stdout monitor: parsed event #{} - type: {}",
            self.event_count,
            event.event_type()
        );

        // Process based on event type
        match &event {
            ClaudeEvent::Init { session_id } => {
                // Capture session ID from init event
                if let Some(sid) = session_id {
                    debug!("Captured session ID from init: {}", sid);
                    self.session_id = Some(sid.clone());
                }
            }
            ClaudeEvent::Assistant { .. } => {
                // Check for promise in text content
                if let Some(text) = event.extract_text() {
                    if self.promise_regex.is_match(&text) {
                        info!(
                            "Promise found in output: {}",
                            self.config.completion_promise
                        );
                        self.state
                            .set_promise_found(self.config.completion_promise.clone())
                            .await;
                    }
                }
            }
            ClaudeEvent::Result {
                session_id, usage, ..
            } => {
                // Capture session ID from result event (may override init)
                if let Some(sid) = session_id {
                    debug!("Captured session ID from result: {}", sid);
                    self.session_id = Some(sid.clone());
                }

                // Capture token usage
                self.token_usage = Some(usage.clone());

                // Update token count with actual usage from Claude
                let total = usage.total();
                debug!("Result event: {} total tokens", total);

                // Set the token count to the actual value
                self.state.set_tokens(total).await;

                // Check for warning threshold
                if !self.warning_emitted && total >= self.config.context_limit.warning_threshold {
                    warn!(
                        "Context limit warning: {} tokens (threshold: {})",
                        total, self.config.context_limit.warning_threshold
                    );
                    self.warning_emitted = true;
                }

                // Check for context limit
                if total >= self.config.context_limit.max_tokens {
                    info!(
                        "Context limit reached: {} tokens (limit: {})",
                        total, self.config.context_limit.max_tokens
                    );
                    let _ = self.cmd_tx.send(ProcessCommand::Kill).await;
                }
            }
            _ => {
                // Other event types - just logged for debugging
                debug!("Event: {:?}", event);
            }
        }

        Ok(())
    }
}

/// Plain text monitor for stderr
pub struct StderrMonitor {
    line_count: u64,
}

impl Default for StderrMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl StderrMonitor {
    /// Create a new StderrMonitor
    pub fn new() -> Self {
        Self { line_count: 0 }
    }

    /// Monitor stderr for plain text output
    pub async fn monitor_stream<R>(&mut self, reader: &mut BufReader<R>) -> crate::error::Result<()>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        debug!("stderr monitor: starting");
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!(
                        "stderr monitor: stream closed after {} lines",
                        self.line_count
                    );
                    break;
                }
                Ok(_) => {
                    self.line_count += 1;
                    // stderr is informational/error messages, just log them
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        debug!("stderr[{}]: {}", self.line_count, trimmed);
                    }
                }
                Err(e) => {
                    warn!(
                        "stderr monitor: read error after {} lines: {}",
                        self.line_count, e
                    );
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Spawn monitor tasks for stdout (JSON) and stderr (plain text)
///
/// Returns handles for both tasks. The stdout handle returns MonitorResult with
/// captured session ID and token usage.
pub fn spawn_monitors(
    config: Arc<Config>,
    state: Arc<SharedState>,
    stdout: BufReader<tokio::process::ChildStdout>,
    stderr: BufReader<tokio::process::ChildStderr>,
    cmd_tx: mpsc::Sender<ProcessCommand>,
) -> (
    tokio::task::JoinHandle<MonitorResult>,
    tokio::task::JoinHandle<()>,
) {
    debug!("spawn_monitors: creating stdout and stderr monitor tasks");
    let config_stdout = Arc::clone(&config);
    let state_stdout = Arc::clone(&state);

    let stdout_handle = tokio::spawn(async move {
        debug!("stdout monitor task: started");
        let mut stdout = stdout;
        let mut monitor = JsonEventMonitor::new(config_stdout, state_stdout, cmd_tx);
        if let Err(e) = monitor.monitor_stream(&mut stdout).await {
            warn!("stdout monitor error: {}", e);
        }
        debug!("stdout monitor task: exiting");
        monitor.result()
    });

    let stderr_handle = tokio::spawn(async move {
        debug!("stderr monitor task: started");
        let mut stderr = stderr;
        let mut monitor = StderrMonitor::new();
        if let Err(e) = monitor.monitor_stream(&mut stderr).await {
            warn!("stderr monitor error: {}", e);
        }
        debug!("stderr monitor task: exiting");
    });

    debug!("spawn_monitors: tasks spawned successfully");
    (stdout_handle, stderr_handle)
}
