use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, trace, warn};

use crate::agent::{Agent, AgentResult, ExitReason};
use crate::config::Config;
use crate::error::{RalphError, Result};
use crate::state::SharedState;
use crate::transcript::{ExitReason as TranscriptExitReason, IterationEndReason, TranscriptWriter};

/// Result of the loop execution
#[derive(Debug, Clone)]
pub enum LoopResult {
    /// The completion promise was found
    PromiseFulfilled {
        /// Number of iterations it took
        iterations: u32,
        /// The promise text that was found
        promise: String,
    },
    /// Shutdown was requested
    Shutdown {
        /// Number of iterations completed before shutdown
        iterations: u32,
    },
}

/// Main loop controller that orchestrates agent invocations
pub struct LoopController<A: Agent> {
    config: Arc<Config>,
    agent: A,
    state: Arc<SharedState>,
    transcript_writer: Option<Arc<Mutex<TranscriptWriter>>>,
}

impl<A: Agent> LoopController<A> {
    /// Create a new LoopController
    pub fn new(config: Config, agent: A) -> Self {
        Self {
            config: Arc::new(config),
            agent,
            state: SharedState::new_shared(),
            transcript_writer: None,
        }
    }

    /// Create a new LoopController with a transcript writer
    pub fn with_transcript_writer(config: Config, agent: A, project_path: &Path) -> Result<Self> {
        let output_dir = &config.output_dir;
        let writer = TranscriptWriter::new(
            output_dir,
            project_path,
            &config.prompt,
            None, // prompt_file not tracked at this level
            config.completion_promise.clone(),
            None, // auto-generate run_id
        )?;

        Ok(Self {
            config: Arc::new(config),
            agent,
            state: SharedState::new_shared(),
            transcript_writer: Some(Arc::new(Mutex::new(writer))),
        })
    }

    /// Create a new LoopController with an existing shared state
    pub fn with_state(config: Config, agent: A, state: Arc<SharedState>) -> Self {
        Self {
            config: Arc::new(config),
            agent,
            state,
            transcript_writer: None,
        }
    }

    /// Get a reference to the shared state
    pub fn state(&self) -> &Arc<SharedState> {
        &self.state
    }

    /// Get a reference to the config
    pub fn config(&self) -> &Arc<Config> {
        &self.config
    }

    /// Run the loop until the promise is found or max iterations is reached
    pub async fn run(&self) -> Result<LoopResult> {
        let prompt = &self.config.prompt;

        loop {
            // Increment iteration
            let iteration = self.state.increment_iteration().await;

            // Check max iterations
            if let Some(max) = self.config.max_iterations {
                if iteration > max {
                    // Complete transcript with max iterations exceeded
                    if let Some(ref writer) = self.transcript_writer {
                        let mut writer = writer.lock().await;
                        if let Err(e) = writer.complete(TranscriptExitReason::MaxIterationsExceeded)
                        {
                            warn!("Failed to complete transcript: {}", e);
                        }
                    }
                    return Err(RalphError::MaxIterationsExceeded(max));
                }
            }

            info!("Starting iteration {}", iteration);
            debug!("Prompt length: {} chars", prompt.len());
            trace!("Prompt: {}", prompt);

            // Start iteration in transcript
            if let Some(ref writer) = self.transcript_writer {
                let mut writer = writer.lock().await;
                if let Err(e) = writer.start_iteration() {
                    warn!("Failed to start transcript iteration: {}", e);
                }
            }

            // Reset state for new iteration
            debug!("Resetting state for new iteration");
            self.state.reset().await;

            // Run the agent
            debug!("Calling agent.run()...");
            let result: AgentResult = self.agent.run(prompt).await?;
            debug!(
                "Agent returned - exit_reason: {:?}, promise_found: {:?}",
                result.exit_reason,
                result.promise_found.is_some()
            );

            // Record session ID if available
            if let Some(ref session_id) = result.session_id {
                if let Some(ref writer) = self.transcript_writer {
                    let mut writer = writer.lock().await;
                    if let Err(e) = writer.set_session_id(session_id.clone()) {
                        warn!("Failed to set session ID: {}", e);
                    }
                }
            }

            // Determine end reason and record it
            let (end_reason, input_tokens, output_tokens) = match result.exit_reason {
                ExitReason::Natural => {
                    if result.is_fulfilled() {
                        (IterationEndReason::PromiseFound, 0, 0)
                    } else {
                        (IterationEndReason::Normal, 0, 0)
                    }
                }
                ExitReason::ContextLimit => (IterationEndReason::ContextLimit, 0, 0),
                ExitReason::Shutdown => (IterationEndReason::Interrupted, 0, 0),
            };

            // Get token usage from result if available
            let (input_tokens, output_tokens) = if let Some(ref usage) = result.token_usage {
                (usage.input_tokens, usage.output_tokens)
            } else {
                (input_tokens, output_tokens)
            };

            // End iteration in transcript
            if let Some(ref writer) = self.transcript_writer {
                let mut writer = writer.lock().await;
                if let Err(e) = writer.end_iteration(end_reason, input_tokens, output_tokens) {
                    warn!("Failed to end transcript iteration: {}", e);
                }
            }

            // Check if promise was found
            if result.is_fulfilled() {
                let promise = result.promise_found.unwrap_or_default();
                info!(
                    "Promise fulfilled after {} iterations: {}",
                    iteration, promise
                );

                // Complete transcript
                if let Some(ref writer) = self.transcript_writer {
                    let mut writer = writer.lock().await;
                    if let Err(e) = writer.complete(TranscriptExitReason::PromiseFulfilled) {
                        warn!("Failed to complete transcript: {}", e);
                    }
                }

                return Ok(LoopResult::PromiseFulfilled {
                    iterations: iteration,
                    promise,
                });
            }

            info!(
                "Iteration {} complete, no promise found. Continuing...",
                iteration
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Mock agent for testing
    struct MockAgent {
        /// Number of calls before returning a promise
        calls_until_promise: AtomicU32,
        /// The promise to return
        promise: String,
    }

    impl MockAgent {
        fn new(calls_until_promise: u32, promise: &str) -> Self {
            Self {
                calls_until_promise: AtomicU32::new(calls_until_promise),
                promise: promise.to_string(),
            }
        }
    }

    #[async_trait]
    impl Agent for MockAgent {
        async fn run(&self, _prompt: &str) -> Result<AgentResult> {
            let remaining = self.calls_until_promise.fetch_sub(1, Ordering::SeqCst);
            if remaining <= 1 {
                Ok(AgentResult::with_promise(&self.promise))
            } else {
                Ok(AgentResult::without_promise())
            }
        }
    }

    /// Mock agent that always fails to find a promise
    struct NeverFindsMockAgent;

    #[async_trait]
    impl Agent for NeverFindsMockAgent {
        async fn run(&self, _prompt: &str) -> Result<AgentResult> {
            Ok(AgentResult::without_promise())
        }
    }

    /// Mock agent that returns a context limit exit
    #[allow(dead_code)]
    struct ContextLimitMockAgent;

    #[async_trait]
    impl Agent for ContextLimitMockAgent {
        async fn run(&self, _prompt: &str) -> Result<AgentResult> {
            Ok(AgentResult {
                output: String::new(),
                promise_found: None,
                token_count: 200_000,
                exit_reason: ExitReason::ContextLimit,
                session_id: None,
                token_usage: None,
            })
        }
    }

    #[tokio::test]
    async fn test_loop_continues_until_promise_fulfilled() {
        let agent = MockAgent::new(3, "TASK COMPLETE");
        let config = Config {
            prompt: "test prompt".to_string(),
            max_iterations: Some(10),
            completion_promise: "TASK COMPLETE".to_string(),
            ..Config::default()
        };

        let controller = LoopController::new(config, agent);
        let result = controller.run().await.unwrap();

        match result {
            LoopResult::PromiseFulfilled {
                iterations,
                promise,
            } => {
                assert_eq!(iterations, 3);
                assert_eq!(promise, "TASK COMPLETE");
            }
            _ => panic!("Expected PromiseFulfilled"),
        }
    }

    #[tokio::test]
    async fn test_loop_stops_on_first_iteration_if_promise_found_immediately() {
        let agent = MockAgent::new(1, "DONE");
        let config = Config {
            prompt: "test prompt".to_string(),
            max_iterations: Some(10),
            completion_promise: "DONE".to_string(),
            ..Config::default()
        };

        let controller = LoopController::new(config, agent);
        let result = controller.run().await.unwrap();

        match result {
            LoopResult::PromiseFulfilled { iterations, .. } => {
                assert_eq!(iterations, 1);
            }
            _ => panic!("Expected PromiseFulfilled"),
        }
    }

    #[tokio::test]
    async fn test_loop_respects_max_iterations_limit() {
        let agent = NeverFindsMockAgent;
        let config = Config {
            prompt: "test prompt".to_string(),
            max_iterations: Some(5),
            ..Config::default()
        };

        let controller = LoopController::new(config, agent);
        let result = controller.run().await;

        match result {
            Err(RalphError::MaxIterationsExceeded(max)) => {
                assert_eq!(max, 5);
            }
            _ => panic!("Expected MaxIterationsExceeded error"),
        }
    }

    #[tokio::test]
    async fn test_returns_max_iterations_exceeded_error() {
        let agent = NeverFindsMockAgent;
        let config = Config {
            prompt: "test prompt".to_string(),
            max_iterations: Some(3),
            ..Config::default()
        };

        let controller = LoopController::new(config, agent);
        let result = controller.run().await;

        assert!(matches!(result, Err(RalphError::MaxIterationsExceeded(3))));
    }
}
