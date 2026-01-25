//! Formatted output display for transcripts.

use std::path::Path;

use colored::Colorize;

use crate::error::{Result, ViewerError};
use crate::run::RunMetadata;
use crate::transcript::{read_transcript, ContentBlock, TranscriptEvent};

/// Display a transcript for a run
pub fn display_transcript(run: &RunMetadata, iteration: Option<u32>) -> Result<()> {
    if run.iterations.is_empty() {
        eprintln!("No iterations found in run {}", run.run_id);
        return Ok(());
    }

    // Show transcript location
    let transcripts_dir = run.claude_transcripts_dir();
    eprintln!(
        "{} {}",
        "Transcripts from:".dimmed(),
        transcripts_dir.display()
    );
    eprintln!();

    match iteration {
        Some(iter) => {
            // Display single iteration (1-indexed to 0-indexed)
            let index = (iter as usize).saturating_sub(1);
            if index >= run.iterations.len() {
                return Err(ViewerError::InvalidIteration(iter));
            }

            print_iteration_header(iter, &run.iterations[index]);
            display_iteration(run, index)?;
        }
        None => {
            // Display all iterations
            for (i, iteration_meta) in run.iterations.iter().enumerate() {
                print_iteration_header((i + 1) as u32, iteration_meta);
                display_iteration(run, i)?;
            }
        }
    }

    Ok(())
}

/// Display a single iteration's transcript
fn display_iteration(run: &RunMetadata, iteration_index: usize) -> Result<()> {
    match run.transcript_path(iteration_index) {
        Some(path) => {
            if path.exists() {
                display_transcript_file(&path)?;
            } else {
                eprintln!(
                    "  {} Transcript file not found: {}",
                    "⚠".yellow(),
                    path.display()
                );
                eprintln!(
                    "    The session transcript may have been cleaned up by Claude Code."
                );
            }
        }
        None => {
            eprintln!("  {} No session ID recorded for this iteration", "⚠".yellow());
        }
    }
    Ok(())
}

/// Display a single transcript file
pub fn display_transcript_file(path: &Path) -> Result<()> {
    let events = read_transcript(path)?;

    for event in events {
        display_event(&event);
    }

    Ok(())
}

/// Display a single event with formatting
pub fn display_event(event: &TranscriptEvent) {
    match event {
        TranscriptEvent::Init { session_id } => {
            if let Some(sid) = session_id {
                println!("{} Session: {}", "▶".cyan(), sid.dimmed());
            }
        }
        TranscriptEvent::User { content } => {
            // User messages - show briefly
            let preview = summarize_content(content, 100);
            if !preview.is_empty() {
                println!("\n{} {}", "👤".blue(), preview.dimmed());
            }
        }
        TranscriptEvent::Assistant { content } => {
            for block in content {
                match block {
                    ContentBlock::Text { text } => {
                        println!("{}", text);
                    }
                    ContentBlock::ToolUse { name, input, .. } => {
                        println!("\n{} {}", "Tool:".yellow().bold(), name.cyan());
                        // Show key inputs
                        if let Some(obj) = input.as_object() {
                            for (key, value) in obj {
                                let display_value = format_input_value(value);
                                if !display_value.is_empty() {
                                    println!("  {}: {}", key.dimmed(), display_value);
                                }
                            }
                        }
                    }
                    ContentBlock::Other => {}
                }
            }
        }
        TranscriptEvent::ToolUse { name, input, .. } => {
            println!("\n{} {}", "Tool:".yellow().bold(), name.cyan());
            if let Some(obj) = input.as_object() {
                for (key, value) in obj {
                    let display_value = format_input_value(value);
                    if !display_value.is_empty() {
                        println!("  {}: {}", key.dimmed(), display_value);
                    }
                }
            }
        }
        TranscriptEvent::ToolResult { content, .. } => {
            // Summarize tool results (they can be very long)
            let preview = summarize_content(content, 200);
            if !preview.is_empty() {
                println!("{} {}", "→".green(), preview.dimmed());
            }
        }
        TranscriptEvent::Result {
            usage,
            total_cost_usd,
            ..
        } => {
            println!();
            println!(
                "{} {} tokens (input: {}, output: {})",
                "⏹".magenta(),
                usage.total().to_string().bold(),
                usage.input_tokens,
                usage.output_tokens
            );
            if let Some(cost) = total_cost_usd {
                println!("  Cost: ${:.4}", cost);
            }
        }
        TranscriptEvent::Unknown { .. } => {
            // Skip unknown events
        }
    }
}

/// Print an iteration header with metadata
fn print_iteration_header(iteration: u32, meta: &crate::run::IterationMetadata) {
    let header = format!("━━━ Iteration {} ", iteration);
    let padding = "━".repeat(60_usize.saturating_sub(header.len()));
    println!("\n{}{}", header.bold(), padding);

    // Show iteration metadata
    if let Some(session_id) = &meta.session_id {
        println!("Session: {}", session_id.dimmed());
    }
    if let Some(tokens) = &meta.tokens {
        println!(
            "Tokens: {} (input: {}, output: {})",
            (tokens.input + tokens.output).to_string().dimmed(),
            tokens.input,
            tokens.output
        );
    }
    if let Some(end_reason) = &meta.end_reason {
        println!("End reason: {:?}", end_reason);
    }
    println!();
}

/// Format an input value for display
fn format_input_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => {
            if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s.clone()
            }
        }
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Array(arr) => format!("[{} items]", arr.len()),
        serde_json::Value::Object(obj) => format!("{{{} keys}}", obj.len()),
    }
}

/// Summarize long content for display
fn summarize_content(content: &str, max_len: usize) -> String {
    let content = content.trim();
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}... ({} chars)", &content[..max_len], content.len())
    }
}
