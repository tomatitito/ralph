use std::sync::Arc;
use tracing::{debug, info};

use crate::agent::{Agent, AgentResult};
use crate::config::Config;
use crate::error::{RalphError, Result};
use crate::state::SharedState;

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
}

impl<A: Agent> LoopController<A> {
    /// Create a new LoopController
    pub fn new(config: Config, agent: A) -> Self {
        Self {
            config: Arc::new(config),
            agent,
            state: SharedState::new_shared(),
        }
    }

    /// Create a new LoopController with an existing shared state
    pub fn with_state(config: Config, agent: A, state: Arc<SharedState>) -> Self {
        Self {
            config: Arc::new(config),
            agent,
            state,
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
                    return Err(RalphError::MaxIterationsExceeded(max));
                }
            }

            info!("Starting iteration {}", iteration);
            debug!("Prompt: {}", prompt);

            // Reset state for new iteration
            self.state.reset().await;

            // Run the agent
            let result: AgentResult = self.agent.run(prompt).await?;

            // Check if promise was found
            if result.is_fulfilled() {
                let promise = result.promise_found.unwrap_or_default();
                info!(
                    "Promise fulfilled after {} iterations: {}",
                    iteration, promise
                );
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
    use crate::agent::ExitReason;
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
