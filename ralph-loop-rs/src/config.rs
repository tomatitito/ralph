use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Token estimation method for context tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenEstimationMethod {
    /// Use tiktoken with cl100k_base encoding (most accurate)
    #[default]
    Tiktoken,
    /// Estimate as text.len() / 4
    ByteRatio,
    /// Estimate as text.chars().count() / 4
    CharRatio,
}

/// Context limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLimitConfig {
    /// Maximum tokens before killing process
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    /// Token count at which to emit a warning
    #[serde(default = "default_warning_threshold")]
    pub warning_threshold: usize,
    /// Method for estimating token count
    #[serde(default)]
    pub estimation_method: TokenEstimationMethod,
}

fn default_max_tokens() -> usize {
    180_000
}

fn default_warning_threshold() -> usize {
    150_000
}

impl Default for ContextLimitConfig {
    fn default() -> Self {
        Self {
            max_tokens: default_max_tokens(),
            warning_threshold: default_warning_threshold(),
            estimation_method: TokenEstimationMethod::default(),
        }
    }
}

/// Main configuration for the ralph-loop application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The prompt to send to Claude
    #[serde(default)]
    pub prompt: String,
    /// Maximum number of iterations (None = infinite loop)
    #[serde(default)]
    pub max_iterations: Option<u32>,
    /// Text to look for to consider the task complete
    #[serde(default = "default_completion_promise")]
    pub completion_promise: String,
    /// Context limit configuration
    #[serde(default)]
    pub context_limit: ContextLimitConfig,
    /// Directory for output files
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    /// Path to the Claude CLI executable
    #[serde(default = "default_claude_path")]
    pub claude_path: String,
    /// Additional arguments to pass to Claude
    #[serde(default = "default_claude_args")]
    pub claude_args: Vec<String>,
}

fn default_completion_promise() -> String {
    "TASK COMPLETE".to_string()
}

fn default_output_dir() -> PathBuf {
    PathBuf::from(".ralph-loop-output")
}

fn default_claude_path() -> String {
    "claude".to_string()
}

fn default_claude_args() -> Vec<String> {
    vec![
        "--print".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--dangerously-skip-permissions".to_string(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            max_iterations: None,
            completion_promise: default_completion_promise(),
            context_limit: ContextLimitConfig::default(),
            output_dir: default_output_dir(),
            claude_path: default_claude_path(),
            claude_args: default_claude_args(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &std::path::Path) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::RalphError::ConfigError(e.to_string()))?;
        toml::from_str(&content).map_err(|e| crate::error::RalphError::ConfigError(e.to_string()))
    }

    /// Merge CLI arguments into this configuration
    /// CLI arguments take precedence over config file values
    pub fn merge_cli_args(
        &mut self,
        prompt: Option<String>,
        max_iterations: Option<u32>,
        completion_promise: Option<String>,
        output_dir: Option<PathBuf>,
        context_limit: Option<usize>,
    ) {
        if let Some(p) = prompt {
            self.prompt = p;
        }
        if max_iterations.is_some() {
            self.max_iterations = max_iterations;
        }
        if let Some(cp) = completion_promise {
            self.completion_promise = cp;
        }
        if let Some(od) = output_dir {
            self.output_dir = od;
        }
        if let Some(cl) = context_limit {
            self.context_limit.max_tokens = cl;
        }
    }
}
