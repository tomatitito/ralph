//! Run discovery and metadata parsing.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Result, ViewerError};

/// Status of a run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
    Interrupted,
}

/// Reason why an iteration ended
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IterationEndReason {
    ContextLimit,
    PromiseFound,
    Normal,
    Interrupted,
    Error,
}

/// Token usage record for an iteration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsageRecord {
    pub input: usize,
    pub output: usize,
}

/// Metadata about a single iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationMetadata {
    pub iteration: u32,
    #[serde(default)]
    pub session_id: Option<String>,
    pub started_at: DateTime<Utc>,
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub end_reason: Option<IterationEndReason>,
    #[serde(default)]
    pub tokens: Option<TokenUsageRecord>,
}

/// Metadata about a run (matches ralph-loop's .ralph-meta.json format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetadata {
    pub run_id: String,
    pub status: RunStatus,
    pub started_at: DateTime<Utc>,
    #[serde(default)]
    pub completed_at: Option<DateTime<Utc>>,
    /// Absolute path to the project
    pub project_path: String,
    #[serde(default)]
    pub prompt_file: Option<String>,
    pub prompt_preview: String,
    pub completion_promise: String,
    #[serde(default)]
    pub exit_reason: Option<String>,
    /// Per-iteration metadata with session ID mappings
    #[serde(default)]
    pub iterations: Vec<IterationMetadata>,
}

impl RunMetadata {
    /// Load metadata from a run directory
    pub fn load(run_dir: &Path) -> Result<Self> {
        let meta_path = run_dir.join(".ralph-meta.json");
        let content = fs::read_to_string(&meta_path)?;
        let metadata: RunMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    /// Get a display string for the status
    pub fn status_display(&self) -> &'static str {
        match self.status {
            RunStatus::Running => "running",
            RunStatus::Completed => "done",
            RunStatus::Failed => "failed",
            RunStatus::Interrupted => "interrupted",
        }
    }

    /// Check if the run is still active
    pub fn is_active(&self) -> bool {
        self.status == RunStatus::Running
    }

    /// Get the current iteration count
    pub fn current_iteration(&self) -> u32 {
        self.iterations.len() as u32
    }

    /// Get total tokens across all iterations
    pub fn total_tokens(&self) -> usize {
        self.iterations
            .iter()
            .filter_map(|i| i.tokens.as_ref())
            .map(|t| t.input + t.output)
            .sum()
    }

    /// Get the Claude Code transcript directory for this project
    pub fn claude_transcripts_dir(&self) -> PathBuf {
        // Convert project path to Claude's format: /home/sprite/ralph -> -home-sprite-ralph
        let sanitized = self.project_path.replace('/', "-");
        // Remove leading dash if present
        let sanitized = sanitized.strip_prefix('-').unwrap_or(&sanitized);

        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(".claude")
            .join("projects")
            .join(sanitized)
    }

    /// Get the transcript file path for a specific iteration (0-indexed)
    pub fn transcript_path(&self, iteration_index: usize) -> Option<PathBuf> {
        let iteration = self.iterations.get(iteration_index)?;
        let session_id = iteration.session_id.as_ref()?;
        let dir = self.claude_transcripts_dir();
        Some(dir.join(format!("{}.jsonl", session_id)))
    }
}

/// Discovers and lists available runs
pub struct RunDiscovery {
    runs_dir: PathBuf,
}

impl RunDiscovery {
    /// Create a new RunDiscovery for the given output directory
    pub fn new(output_dir: &Path) -> Result<Self> {
        let runs_dir = output_dir.join("runs");
        if !runs_dir.exists() {
            return Err(ViewerError::NoRunsAvailable);
        }
        Ok(Self { runs_dir })
    }

    /// List all available runs, sorted by start time (newest first)
    pub fn list_runs(&self) -> Result<Vec<RunMetadata>> {
        let mut runs = Vec::new();

        for entry in fs::read_dir(&self.runs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                match RunMetadata::load(&path) {
                    Ok(metadata) => runs.push(metadata),
                    Err(e) => {
                        // Skip invalid run directories
                        eprintln!(
                            "Warning: skipping invalid run directory {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        // Sort by start time (newest first)
        runs.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(runs)
    }

    /// Get a specific run by ID
    pub fn get_run(&self, run_id: &str) -> Result<RunMetadata> {
        let run_dir = self.runs_dir.join(run_id);
        if !run_dir.exists() {
            return Err(ViewerError::RunNotFound(run_id.to_string()));
        }
        RunMetadata::load(&run_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_run_status_display() {
        let metadata = RunMetadata {
            run_id: "test".to_string(),
            status: RunStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            project_path: "/home/test".to_string(),
            prompt_file: None,
            prompt_preview: "test".to_string(),
            completion_promise: "DONE".to_string(),
            exit_reason: None,
            iterations: vec![],
        };
        assert_eq!(metadata.status_display(), "running");
    }

    #[test]
    fn test_run_discovery_empty() {
        let temp_dir = TempDir::new().unwrap();
        let runs_dir = temp_dir.path().join("runs");
        fs::create_dir_all(&runs_dir).unwrap();

        let discovery = RunDiscovery::new(temp_dir.path()).unwrap();
        let runs = discovery.list_runs().unwrap();
        assert!(runs.is_empty());
    }

    #[test]
    fn test_claude_transcripts_dir() {
        let metadata = RunMetadata {
            run_id: "test".to_string(),
            status: RunStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            project_path: "/home/sprite/ralph".to_string(),
            prompt_file: None,
            prompt_preview: "test".to_string(),
            completion_promise: "DONE".to_string(),
            exit_reason: None,
            iterations: vec![],
        };

        let transcripts_dir = metadata.claude_transcripts_dir();
        // Should end with the sanitized project path
        assert!(transcripts_dir
            .to_string_lossy()
            .contains("home-sprite-ralph"));
    }

    #[test]
    fn test_transcript_path() {
        let metadata = RunMetadata {
            run_id: "test".to_string(),
            status: RunStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            project_path: "/home/sprite/ralph".to_string(),
            prompt_file: None,
            prompt_preview: "test".to_string(),
            completion_promise: "DONE".to_string(),
            exit_reason: None,
            iterations: vec![IterationMetadata {
                iteration: 1,
                session_id: Some("abc123".to_string()),
                started_at: Utc::now(),
                ended_at: None,
                end_reason: None,
                tokens: None,
            }],
        };

        let path = metadata.transcript_path(0).unwrap();
        assert!(path.to_string_lossy().ends_with("abc123.jsonl"));
    }

    #[test]
    fn test_total_tokens() {
        let metadata = RunMetadata {
            run_id: "test".to_string(),
            status: RunStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            project_path: "/home/test".to_string(),
            prompt_file: None,
            prompt_preview: "test".to_string(),
            completion_promise: "DONE".to_string(),
            exit_reason: None,
            iterations: vec![
                IterationMetadata {
                    iteration: 1,
                    session_id: Some("sess1".to_string()),
                    started_at: Utc::now(),
                    ended_at: None,
                    end_reason: None,
                    tokens: Some(TokenUsageRecord {
                        input: 1000,
                        output: 500,
                    }),
                },
                IterationMetadata {
                    iteration: 2,
                    session_id: Some("sess2".to_string()),
                    started_at: Utc::now(),
                    ended_at: None,
                    end_reason: None,
                    tokens: Some(TokenUsageRecord {
                        input: 2000,
                        output: 1000,
                    }),
                },
            ],
        };

        assert_eq!(metadata.total_tokens(), 4500);
        assert_eq!(metadata.current_iteration(), 2);
    }
}
