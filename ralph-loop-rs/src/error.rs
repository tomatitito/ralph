use thiserror::Error;

/// Errors that can occur in the Ralph Loop application
#[derive(Error, Debug)]
pub enum RalphError {
    /// Maximum iterations reached without finding the promise
    #[error("maximum iterations ({0}) exceeded without finding promise")]
    MaxIterationsExceeded(u32),

    /// Shutdown was requested (e.g., via Ctrl+C)
    #[error("shutdown requested")]
    ShutdownRequested,

    /// Failed to spawn the Claude subprocess
    #[error("failed to spawn Claude process: {0}")]
    ProcessSpawnError(#[source] std::io::Error),

    /// Error communicating with the Claude subprocess
    #[error("process I/O error: {0}")]
    ProcessIoError(#[source] std::io::Error),

    /// Error reading or parsing configuration
    #[error("configuration error: {0}")]
    ConfigError(String),

    /// Error reading prompt file
    #[error("failed to read prompt file: {0}")]
    PromptFileError(#[source] std::io::Error),

    /// No prompt provided
    #[error("no prompt provided: use -p or -f to specify a prompt")]
    NoPromptProvided,

    /// Error creating output directory
    #[error("failed to create output directory: {0}")]
    OutputDirError(#[source] std::io::Error),

    /// Error writing transcript files
    #[error("transcript write error: {0}")]
    TranscriptWriteError(String),

    /// Error parsing JSON from Claude's output
    #[error("JSON parse error: {0}")]
    JsonParseError(String),

    /// Tmux is not available on this system
    #[error("tmux is not available on this system")]
    TmuxNotAvailable,

    /// Error with tmux operations
    #[error("tmux error: {0}")]
    TmuxError(String),
}

/// Result type alias for Ralph operations
pub type Result<T> = std::result::Result<T, RalphError>;
