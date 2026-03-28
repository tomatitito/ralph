use clap::ValueEnum;
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

/// Supported coding agent backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum AgentProvider {
    /// Anthropic Claude Code CLI
    #[default]
    Claude,
    /// OpenAI Codex CLI
    Codex,
}

/// Agent execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Which coding agent backend to invoke
    #[serde(default)]
    pub provider: AgentProvider,
    /// Path to the agent CLI executable
    #[serde(default)]
    pub path: Option<String>,
    /// Additional arguments to pass to the agent CLI
    #[serde(default)]
    pub args: Option<Vec<String>>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: AgentProvider::Claude,
            path: None,
            args: None,
        }
    }
}

/// CLI-provided config overrides
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub prompt: Option<String>,
    pub max_iterations: Option<u32>,
    pub completion_promise: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub context_limit: Option<usize>,
    pub agent_provider: Option<AgentProvider>,
    pub agent_path: Option<String>,
    pub agent_args: Option<Vec<String>>,
}

/// Main configuration for the ralph-loop application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The prompt to send to the configured coding agent
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
    /// Coding agent execution settings
    #[serde(default)]
    pub agent: AgentConfig,
    /// Legacy Claude CLI path setting kept for backward compatibility
    #[serde(default)]
    pub claude_path: Option<String>,
    /// Legacy Claude CLI args kept for backward compatibility
    #[serde(default)]
    pub claude_args: Option<Vec<String>>,
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

fn default_codex_path() -> String {
    "codex".to_string()
}

fn default_claude_args() -> Vec<String> {
    vec![
        "--print".to_string(),
        "--verbose".to_string(),
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
            agent: AgentConfig::default(),
            claude_path: None,
            claude_args: None,
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &std::path::Path) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::RalphError::ConfigError(e.to_string()))?;
        let mut config: Self = toml::from_str(&content)
            .map_err(|e| crate::error::RalphError::ConfigError(e.to_string()))?;
        config.apply_legacy_defaults();
        Ok(config)
    }

    /// Merge CLI arguments into this configuration
    /// CLI arguments take precedence over config file values
    pub fn merge_cli_args(&mut self, overrides: CliOverrides) {
        if let Some(p) = overrides.prompt {
            self.prompt = p;
        }
        if overrides.max_iterations.is_some() {
            self.max_iterations = overrides.max_iterations;
        }
        if let Some(cp) = overrides.completion_promise {
            self.completion_promise = cp;
        }
        if let Some(od) = overrides.output_dir {
            self.output_dir = od;
        }
        if let Some(cl) = overrides.context_limit {
            self.context_limit.max_tokens = cl;
        }
        if let Some(provider) = overrides.agent_provider {
            self.agent.provider = provider;
        }
        if let Some(path) = overrides.agent_path {
            self.agent.path = Some(path);
        }
        if let Some(args) = overrides.agent_args {
            self.agent.args = Some(args);
        }
        self.apply_legacy_defaults();
    }

    /// The effective configured agent provider
    pub fn agent_provider(&self) -> AgentProvider {
        self.agent.provider
    }

    /// The effective configured agent executable path
    pub fn agent_path(&self) -> String {
        self.agent
            .path
            .clone()
            .or_else(|| self.claude_path.clone())
            .unwrap_or_else(|| match self.agent.provider {
                AgentProvider::Claude => default_claude_path(),
                AgentProvider::Codex => default_codex_path(),
            })
    }

    /// The effective configured agent CLI arguments
    pub fn agent_args(&self) -> Vec<String> {
        if let Some(args) = self.agent.args.clone() {
            return args;
        }
        if let Some(args) = self.claude_args.clone() {
            return args;
        }
        match self.agent.provider {
            AgentProvider::Claude => default_claude_args(),
            AgentProvider::Codex => default_codex_args(),
        }
    }

    fn apply_legacy_defaults(&mut self) {
        if self.agent.path.is_none() {
            self.agent.path = self.claude_path.clone();
        }
        if self.agent.args.is_none() {
            self.agent.args = self.claude_args.clone();
        }
    }
}

fn default_codex_args() -> Vec<String> {
    vec![
        "exec".to_string(),
        "--json".to_string(),
        "--dangerously-bypass-approvals-and-sandbox".to_string(),
        "-".to_string(),
    ]
}
