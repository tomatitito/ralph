use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for concurrent access between the loop controller and monitors
#[derive(Debug)]
pub struct SharedState {
    /// Current estimated token count
    pub token_count: RwLock<usize>,
    /// Accumulated output from the agent
    pub output_buffer: RwLock<String>,
    /// Whether the completion promise has been found
    pub promise_found: RwLock<bool>,
    /// The promise text if found
    pub promise_text: RwLock<Option<String>>,
    /// Current iteration number
    pub iteration: RwLock<u32>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedState {
    /// Create a new SharedState with default values
    pub fn new() -> Self {
        Self {
            token_count: RwLock::new(0),
            output_buffer: RwLock::new(String::new()),
            promise_found: RwLock::new(false),
            promise_text: RwLock::new(None),
            iteration: RwLock::new(0),
        }
    }

    /// Create an Arc-wrapped SharedState for sharing between tasks
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Reset the state for a new iteration
    pub async fn reset(&self) {
        *self.token_count.write().await = 0;
        *self.output_buffer.write().await = String::new();
        *self.promise_found.write().await = false;
        *self.promise_text.write().await = None;
    }

    /// Increment the iteration counter
    pub async fn increment_iteration(&self) -> u32 {
        let mut iter = self.iteration.write().await;
        *iter += 1;
        *iter
    }

    /// Get the current token count
    pub async fn get_token_count(&self) -> usize {
        *self.token_count.read().await
    }

    /// Add to the token count
    pub async fn add_tokens(&self, count: usize) {
        *self.token_count.write().await += count;
    }

    /// Set the token count to a specific value
    pub async fn set_tokens(&self, count: usize) {
        *self.token_count.write().await = count;
    }

    /// Check if the promise has been found
    pub async fn is_promise_found(&self) -> bool {
        *self.promise_found.read().await
    }

    /// Set the promise as found with the given text
    pub async fn set_promise_found(&self, text: String) {
        *self.promise_found.write().await = true;
        *self.promise_text.write().await = Some(text);
    }

    /// Get the promise text if found
    pub async fn get_promise_text(&self) -> Option<String> {
        self.promise_text.read().await.clone()
    }

    /// Append text to the output buffer
    pub async fn append_output(&self, text: &str) {
        self.output_buffer.write().await.push_str(text);
    }

    /// Get the current output buffer
    pub async fn get_output(&self) -> String {
        self.output_buffer.read().await.clone()
    }
}
