//! Live file watching for transcript files.

use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::time::Duration;

use colored::Colorize;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::{Result, ViewerError};
use crate::formatter::{display_event, display_transcript_file};
use crate::run::RunMetadata;
use crate::transcript::TranscriptEvent;

/// Watch a transcript and display new events as they arrive
pub async fn watch_transcript(run: &RunMetadata, iteration: Option<u32>) -> Result<()> {
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

    // Determine which iteration(s) to watch
    let iteration_index = match iteration {
        Some(iter) => (iter as usize).saturating_sub(1),
        None => run.iterations.len().saturating_sub(1), // Watch latest
    };

    if iteration_index >= run.iterations.len() {
        return Err(ViewerError::InvalidIteration(iteration.unwrap_or(0)));
    }

    let file_path = match run.transcript_path(iteration_index) {
        Some(path) => path,
        None => {
            eprintln!(
                "{} No session ID recorded for iteration {}",
                "⚠".yellow(),
                iteration_index + 1
            );
            return Ok(());
        }
    };

    if !file_path.exists() {
        eprintln!(
            "{} Transcript file not found: {}",
            "⚠".yellow(),
            file_path.display()
        );
        eprintln!("The session transcript may have been cleaned up by Claude Code.");
        return Ok(());
    }

    // Print iteration header
    let iteration_meta = &run.iterations[iteration_index];
    print_iteration_header((iteration_index + 1) as u32, iteration_meta);

    // Display existing content first
    display_transcript_file(&file_path)?;

    // Then watch for changes
    watch_file_for_changes(&file_path).await
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

/// Watch a file for changes and display new events
async fn watch_file_for_changes(file_path: &PathBuf) -> Result<()> {
    // Track current file position
    let mut file = File::open(file_path)?;
    let mut current_pos = file.seek(SeekFrom::End(0))?;

    // Set up file watcher
    let (tx, rx) = std_mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_millis(100)),
    )
    .map_err(|e| ViewerError::WatcherError(e.to_string()))?;

    watcher
        .watch(file_path, RecursiveMode::NonRecursive)
        .map_err(|e| ViewerError::WatcherError(e.to_string()))?;

    println!("\n--- Watching for new events (Ctrl+C to exit) ---\n");

    // Also set up ctrl+c handler
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();

    ctrlc_handler(r);

    // Poll for changes
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // Check for file events with timeout
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(_event) => {
                // File changed, read new content
                read_new_events(&mut file, &mut current_pos)?;
            }
            Err(std_mpsc::RecvTimeoutError::Timeout) => {
                // No events, just continue
            }
            Err(std_mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    println!("\n--- Stopped watching ---");
    Ok(())
}

/// Read and display new events from a file
fn read_new_events(file: &mut File, current_pos: &mut u64) -> Result<()> {
    // Seek to current position
    file.seek(SeekFrom::Start(*current_pos))?;

    let reader = BufReader::new(file.try_clone()?);
    let mut new_pos = *current_pos;

    for line in reader.lines() {
        let line = line?;
        new_pos += line.len() as u64 + 1; // +1 for newline

        if let Some(event) = TranscriptEvent::parse(&line) {
            display_event(&event);
        }
    }

    *current_pos = new_pos;
    Ok(())
}

/// Set up ctrl+c handler
fn ctrlc_handler(running: std::sync::Arc<std::sync::atomic::AtomicBool>) {
    let _ = ctrlc::set_handler(move || {
        running.store(false, std::sync::atomic::Ordering::SeqCst);
    });
}
