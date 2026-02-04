//! Run discovery and metadata parsing.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Result, ViewerError};

/// Format a duration for human-readable display
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    if total_seconds < 0 {
        return "0s".to_string();
    }

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format a token count for display (with K/M suffixes)
pub fn format_tokens(tokens: usize) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

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

    /// Get the duration of the run
    pub fn duration(&self) -> chrono::Duration {
        let end = self.completed_at.unwrap_or_else(Utc::now);
        end.signed_duration_since(self.started_at)
    }

    /// Format the duration for display
    pub fn duration_display(&self) -> String {
        format_duration(self.duration())
    }

    /// Get a human-readable exit reason
    pub fn exit_reason_display(&self) -> &str {
        if self.is_active() {
            return "in progress";
        }

        // Check exit_reason field first
        if let Some(ref reason) = self.exit_reason {
            return match reason.as_str() {
                "promise_fulfilled" => "promise found",
                "max_iterations_exceeded" => "max iterations",
                "interrupted" => "interrupted",
                "error" => "error",
                _ => reason.as_str(),
            };
        }

        // Infer from status and last iteration
        match self.status {
            RunStatus::Completed => {
                // Check if promise was found in last iteration
                if let Some(last) = self.iterations.last() {
                    if let Some(ref end_reason) = last.end_reason {
                        return match end_reason {
                            IterationEndReason::PromiseFound => "promise found",
                            IterationEndReason::ContextLimit => "context limit",
                            IterationEndReason::Normal => "completed",
                            IterationEndReason::Interrupted => "interrupted",
                            IterationEndReason::Error => "error",
                        };
                    }
                }
                "completed"
            }
            RunStatus::Failed => "error",
            RunStatus::Interrupted => "interrupted",
            RunStatus::Running => "in progress",
        }
    }

    /// Get total input tokens across all iterations
    pub fn total_input_tokens(&self) -> usize {
        self.iterations
            .iter()
            .filter_map(|i| i.tokens.as_ref())
            .map(|t| t.input)
            .sum()
    }

    /// Get total output tokens across all iterations
    pub fn total_output_tokens(&self) -> usize {
        self.iterations
            .iter()
            .filter_map(|i| i.tokens.as_ref())
            .map(|t| t.output)
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

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(Duration::seconds(45)), "45s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(Duration::seconds(125)), "2m 5s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(Duration::seconds(3725)), "1h 2m 5s");
    }

    #[test]
    fn test_format_duration_negative() {
        assert_eq!(format_duration(Duration::seconds(-10)), "0s");
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(1500), "1.5K");
        assert_eq!(format_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn test_exit_reason_display_running() {
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
        assert_eq!(metadata.exit_reason_display(), "in progress");
    }

    #[test]
    fn test_exit_reason_display_promise_found() {
        let metadata = RunMetadata {
            run_id: "test".to_string(),
            status: RunStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            project_path: "/home/test".to_string(),
            prompt_file: None,
            prompt_preview: "test".to_string(),
            completion_promise: "DONE".to_string(),
            exit_reason: Some("promise_fulfilled".to_string()),
            iterations: vec![],
        };
        assert_eq!(metadata.exit_reason_display(), "promise found");
    }

    #[test]
    fn test_exit_reason_display_from_iteration() {
        let metadata = RunMetadata {
            run_id: "test".to_string(),
            status: RunStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            project_path: "/home/test".to_string(),
            prompt_file: None,
            prompt_preview: "test".to_string(),
            completion_promise: "DONE".to_string(),
            exit_reason: None,
            iterations: vec![IterationMetadata {
                iteration: 1,
                session_id: None,
                started_at: Utc::now(),
                ended_at: Some(Utc::now()),
                end_reason: Some(IterationEndReason::ContextLimit),
                tokens: None,
            }],
        };
        assert_eq!(metadata.exit_reason_display(), "context limit");
    }

    #[test]
    fn test_duration_display() {
        let started = Utc::now() - Duration::seconds(125);
        let metadata = RunMetadata {
            run_id: "test".to_string(),
            status: RunStatus::Completed,
            started_at: started,
            completed_at: Some(Utc::now()),
            project_path: "/home/test".to_string(),
            prompt_file: None,
            prompt_preview: "test".to_string(),
            completion_promise: "DONE".to_string(),
            exit_reason: None,
            iterations: vec![],
        };
        assert_eq!(metadata.duration_display(), "2m 5s");
    }
}
