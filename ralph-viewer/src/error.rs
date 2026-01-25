//! Error types for the ralph-viewer application.

use thiserror::Error;

/// Errors that can occur in the ralph-viewer application
#[derive(Error, Debug)]
pub enum ViewerError {
    /// Failed to read a file
    #[error("failed to read file: {0}")]
    FileReadError(#[from] std::io::Error),

    /// Failed to parse JSON
    #[error("JSON parse error: {0}")]
    JsonParseError(#[from] serde_json::Error),

    /// Run not found
    #[error("run not found: {0}")]
    RunNotFound(String),

    /// No runs available
    #[error("no runs available")]
    NoRunsAvailable,

    /// User cancelled selection
    #[error("user cancelled")]
    UserCancelled,

    /// File watcher error
    #[error("file watcher error: {0}")]
    WatcherError(String),

    /// Invalid iteration number
    #[error("invalid iteration: {0}")]
    InvalidIteration(u32),
}

/// Result type alias for viewer operations
pub type Result<T> = std::result::Result<T, ViewerError>;
