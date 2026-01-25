//! Transcript file management for ralph-loop.
//!
//! This module handles:
//! - Run directory creation and management
//! - Run ID generation
//! - Symlink management (latest, current)
//! - Transcript file writing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{RalphError, Result};

/// Status of a run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// Run is currently active
    Running,
    /// Run completed successfully (promise found)
    Completed,
    /// Run failed (max iterations, error, etc.)
    Failed,
    /// Run was interrupted (Ctrl+C)
    Interrupted,
}

/// Reason why a run ended
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    /// Completion promise was found
    PromiseFulfilled,
    /// Max iterations exceeded
    MaxIterationsExceeded,
    /// User interrupted (Ctrl+C)
    UserInterrupt,
    /// Context limit reached on final iteration
    ContextLimit,
    /// An error occurred
    Error,
}

/// Reason why an iteration ended
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IterationEndReason {
    /// Context limit reached
    ContextLimit,
    /// Promise was found
    PromiseFound,
    /// Process exited normally
    Normal,
    /// Process was interrupted
    Interrupted,
    /// Error occurred
    Error,
}

/// Metadata about a single iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationMetadata {
    /// Iteration number (1-indexed)
    pub iteration: u32,
    /// Claude Code session ID
    pub session_id: Option<String>,
    /// When this iteration started
    pub started_at: DateTime<Utc>,
    /// When this iteration ended
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    /// Why this iteration ended
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_reason: Option<IterationEndReason>,
    /// Token usage for this iteration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokenUsageRecord>,
}

/// Token usage record for an iteration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsageRecord {
    pub input: usize,
    pub output: usize,
}

/// Metadata about a run stored in .ralph-meta.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetadata {
    /// Unique run identifier
    pub run_id: String,
    /// Current status of the run
    pub status: RunStatus,
    /// When the run started
    pub started_at: DateTime<Utc>,
    /// When the run completed (if finished)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    /// Absolute path to the project
    pub project_path: String,
    /// Path to the prompt file (if used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_file: Option<String>,
    /// First 100 characters of the prompt
    pub prompt_preview: String,
    /// The completion promise being looked for
    pub completion_promise: String,
    /// Why the run ended (if finished)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_reason: Option<ExitReason>,
    /// Per-iteration metadata with session ID mappings
    pub iterations: Vec<IterationMetadata>,
}

impl RunMetadata {
    /// Create new run metadata
    pub fn new(
        run_id: String,
        project_path: String,
        prompt: &str,
        prompt_file: Option<String>,
        completion_promise: String,
    ) -> Self {
        let prompt_preview = if prompt.len() > 100 {
            format!("{}...", &prompt[..100])
        } else {
            prompt.to_string()
        };

        Self {
            run_id,
            status: RunStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            project_path,
            prompt_file,
            prompt_preview,
            completion_promise,
            exit_reason: None,
            iterations: Vec::new(),
        }
    }

    /// Get the current iteration number
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
}

/// Manages run metadata for a single run.
///
/// Note: This writer no longer writes transcript files (iteration_NNN.jsonl).
/// Claude Code stores transcripts at ~/.claude/projects/<project-path>/<session-id>.jsonl
/// and we only store metadata with session ID mappings.
pub struct TranscriptWriter {
    /// Base output directory (.ralph-loop-output)
    output_dir: PathBuf,
    /// Run directory (.ralph-loop-output/runs/<run-id>)
    run_dir: PathBuf,
    /// Run metadata
    metadata: RunMetadata,
}

impl TranscriptWriter {
    /// Create a new TranscriptWriter for a run
    pub fn new(
        output_dir: &Path,
        project_path: &Path,
        prompt: &str,
        prompt_file: Option<String>,
        completion_promise: String,
        run_id: Option<String>,
    ) -> Result<Self> {
        // Generate run ID if not provided
        let run_id = run_id.unwrap_or_else(generate_run_id);

        // Create directory structure
        let runs_dir = output_dir.join("runs");
        let run_dir = runs_dir.join(&run_id);
        fs::create_dir_all(&run_dir).map_err(RalphError::OutputDirError)?;

        // Get absolute project path
        let project_path_str = project_path
            .canonicalize()
            .unwrap_or_else(|_| project_path.to_path_buf())
            .to_string_lossy()
            .to_string();

        // Create metadata
        let metadata = RunMetadata::new(
            run_id,
            project_path_str,
            prompt,
            prompt_file,
            completion_promise,
        );

        let writer = Self {
            output_dir: output_dir.to_path_buf(),
            run_dir,
            metadata,
        };

        // Write initial metadata
        writer.write_metadata()?;

        // Update latest symlink
        writer.update_latest_symlink()?;

        Ok(writer)
    }

    /// Get the run ID
    pub fn run_id(&self) -> &str {
        &self.metadata.run_id
    }

    /// Get the run directory path
    pub fn run_dir(&self) -> &Path {
        &self.run_dir
    }

    /// Start a new iteration
    pub fn start_iteration(&mut self) -> Result<u32> {
        let iteration_num = self.metadata.iterations.len() as u32 + 1;

        let iteration = IterationMetadata {
            iteration: iteration_num,
            session_id: None,
            started_at: Utc::now(),
            ended_at: None,
            end_reason: None,
            tokens: None,
        };

        self.metadata.iterations.push(iteration);
        self.write_metadata()?;

        Ok(iteration_num)
    }

    /// Set the session ID for the current iteration
    pub fn set_session_id(&mut self, session_id: String) -> Result<()> {
        if let Some(iteration) = self.metadata.iterations.last_mut() {
            iteration.session_id = Some(session_id);
            self.write_metadata()?;
        }
        Ok(())
    }

    /// End the current iteration with the given reason and token usage
    pub fn end_iteration(
        &mut self,
        end_reason: IterationEndReason,
        input_tokens: usize,
        output_tokens: usize,
    ) -> Result<()> {
        if let Some(iteration) = self.metadata.iterations.last_mut() {
            iteration.ended_at = Some(Utc::now());
            iteration.end_reason = Some(end_reason);
            iteration.tokens = Some(TokenUsageRecord {
                input: input_tokens,
                output: output_tokens,
            });
            self.write_metadata()?;
        }
        Ok(())
    }

    /// Mark the run as completed
    pub fn complete(&mut self, exit_reason: ExitReason) -> Result<()> {
        self.metadata.status = match exit_reason {
            ExitReason::PromiseFulfilled => RunStatus::Completed,
            ExitReason::UserInterrupt => RunStatus::Interrupted,
            _ => RunStatus::Failed,
        };
        self.metadata.completed_at = Some(Utc::now());
        self.metadata.exit_reason = Some(exit_reason);

        self.write_metadata()
    }

    /// Get a reference to the metadata
    pub fn metadata(&self) -> &RunMetadata {
        &self.metadata
    }

    /// Write metadata to .ralph-meta.json
    fn write_metadata(&self) -> Result<()> {
        let meta_path = self.run_dir.join(".ralph-meta.json");
        let json = serde_json::to_string_pretty(&self.metadata)
            .map_err(|e| RalphError::TranscriptWriteError(e.to_string()))?;
        fs::write(&meta_path, json).map_err(|e| RalphError::TranscriptWriteError(e.to_string()))
    }

    /// Update the 'latest' symlink to point to this run
    fn update_latest_symlink(&self) -> Result<()> {
        let latest_link = self.output_dir.join("latest");

        // Remove existing symlink if present
        if latest_link.exists() || latest_link.is_symlink() {
            let _ = fs::remove_file(&latest_link);
        }

        // Create relative symlink: latest -> runs/<run-id>
        let target = Path::new("runs").join(&self.metadata.run_id);

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, &latest_link)
                .map_err(|e| RalphError::TranscriptWriteError(e.to_string()))?;
        }

        #[cfg(windows)]
        {
            // On Windows, use junction for directory symlink
            std::os::windows::fs::symlink_dir(&target, &latest_link)
                .map_err(|e| RalphError::TranscriptWriteError(e.to_string()))?;
        }

        Ok(())
    }
}

/// Generate a unique run ID in format: YYYYMMDD-HHMMSS-<short-uuid>
pub fn generate_run_id() -> String {
    let now = Utc::now();
    let uuid_short = &Uuid::new_v4().to_string()[..8];
    format!("{}-{}", now.format("%Y%m%d-%H%M%S"), uuid_short)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_run_id_format() {
        let run_id = generate_run_id();
        // Format: YYYYMMDD-HHMMSS-xxxxxxxx
        assert!(run_id.len() >= 23, "Run ID too short: {}", run_id);
        let parts: Vec<&str> = run_id.split('-').collect();
        assert_eq!(parts.len(), 3, "Expected 3 parts in run ID: {}", run_id);
    }

    #[test]
    fn test_transcript_writer_creates_structure() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();
        let project_path = temp_dir.path();

        let writer = TranscriptWriter::new(
            output_dir,
            project_path,
            "Test prompt",
            None,
            "TASK COMPLETE".to_string(),
            Some("test-run-123".to_string()),
        )
        .unwrap();

        // Check run directory exists
        assert!(writer.run_dir().exists());
        assert!(writer.run_dir().join(".ralph-meta.json").exists());

        // Check latest symlink
        let latest = output_dir.join("latest");
        assert!(latest.is_symlink() || latest.exists());
    }

    #[test]
    fn test_transcript_writer_starts_iteration() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();
        let project_path = temp_dir.path();

        let mut writer = TranscriptWriter::new(
            output_dir,
            project_path,
            "Test prompt",
            None,
            "TASK COMPLETE".to_string(),
            Some("test-run-456".to_string()),
        )
        .unwrap();

        let iter = writer.start_iteration().unwrap();
        assert_eq!(iter, 1);

        // Check metadata has iteration
        assert_eq!(writer.metadata().iterations.len(), 1);
        assert_eq!(writer.metadata().iterations[0].iteration, 1);
        assert!(writer.metadata().iterations[0].session_id.is_none());
    }

    #[test]
    fn test_transcript_writer_sets_session_id() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();
        let project_path = temp_dir.path();

        let mut writer = TranscriptWriter::new(
            output_dir,
            project_path,
            "Test prompt",
            None,
            "TASK COMPLETE".to_string(),
            Some("test-run-789".to_string()),
        )
        .unwrap();

        writer.start_iteration().unwrap();
        writer.set_session_id("session-abc123".to_string()).unwrap();

        // Check session ID was set
        assert_eq!(
            writer.metadata().iterations[0].session_id,
            Some("session-abc123".to_string())
        );

        // Read back from file and verify
        let content = fs::read_to_string(writer.run_dir().join(".ralph-meta.json")).unwrap();
        assert!(content.contains("session-abc123"));
    }

    #[test]
    fn test_transcript_writer_ends_iteration() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();
        let project_path = temp_dir.path();

        let mut writer = TranscriptWriter::new(
            output_dir,
            project_path,
            "Test prompt",
            None,
            "TASK COMPLETE".to_string(),
            Some("test-run-end".to_string()),
        )
        .unwrap();

        writer.start_iteration().unwrap();
        writer.set_session_id("session-xyz".to_string()).unwrap();
        writer
            .end_iteration(IterationEndReason::ContextLimit, 1000, 500)
            .unwrap();

        let iteration = &writer.metadata().iterations[0];
        assert!(iteration.ended_at.is_some());
        assert_eq!(iteration.end_reason, Some(IterationEndReason::ContextLimit));
        assert_eq!(iteration.tokens.as_ref().unwrap().input, 1000);
        assert_eq!(iteration.tokens.as_ref().unwrap().output, 500);
    }

    #[test]
    fn test_run_metadata_serialization() {
        let metadata = RunMetadata::new(
            "test-run".to_string(),
            "/home/test/project".to_string(),
            "A long prompt that is over 100 characters. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor.",
            Some("task.txt".to_string()),
            "DONE".to_string(),
        );

        assert_eq!(metadata.status, RunStatus::Running);
        assert!(metadata.prompt_preview.ends_with("..."));
        assert!(metadata.prompt_preview.len() <= 103); // 100 + "..."
        assert_eq!(metadata.project_path, "/home/test/project");
        assert!(metadata.iterations.is_empty());

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        assert!(json.contains("running"));
        assert!(json.contains("DONE"));
        assert!(json.contains("/home/test/project"));
    }

    #[test]
    fn test_run_metadata_total_tokens() {
        let mut metadata = RunMetadata::new(
            "test-run".to_string(),
            "/project".to_string(),
            "prompt",
            None,
            "DONE".to_string(),
        );

        // No iterations = 0 tokens
        assert_eq!(metadata.total_tokens(), 0);

        // Add iterations with tokens
        metadata.iterations.push(IterationMetadata {
            iteration: 1,
            session_id: Some("sess1".to_string()),
            started_at: Utc::now(),
            ended_at: None,
            end_reason: None,
            tokens: Some(TokenUsageRecord {
                input: 1000,
                output: 500,
            }),
        });

        metadata.iterations.push(IterationMetadata {
            iteration: 2,
            session_id: Some("sess2".to_string()),
            started_at: Utc::now(),
            ended_at: None,
            end_reason: None,
            tokens: Some(TokenUsageRecord {
                input: 2000,
                output: 1000,
            }),
        });

        assert_eq!(metadata.total_tokens(), 4500); // 1500 + 3000
    }
}
