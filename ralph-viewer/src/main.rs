use std::path::PathBuf;

use clap::Parser;

mod error;
mod formatter;
mod picker;
mod run;
mod summary;
mod transcript;
mod watcher;

use error::Result;
use run::RunDiscovery;

/// Ralph Viewer: View ralph-loop transcripts
#[derive(Parser, Debug)]
#[command(name = "ralph-viewer")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Output directory to scan (default: .ralph-loop-output)
    #[arg(short = 'd', long = "dir")]
    output_dir: Option<PathBuf>,

    /// Specific run ID to view
    #[arg(short = 'r', long = "run")]
    run: Option<String>,

    /// Specific iteration to view (1-indexed)
    #[arg(short = 'i', long = "iteration")]
    iteration: Option<u32>,

    /// Don't follow live updates (just display and exit)
    #[arg(long = "no-follow")]
    no_follow: bool,

    /// List all runs in simple format
    #[arg(short = 'l', long = "list")]
    list: bool,

    /// Show detailed summary of all runs (default when no run specified)
    #[arg(short = 's', long = "summary")]
    summary: bool,
}

fn default_output_dir() -> PathBuf {
    PathBuf::from(".ralph-loop-output")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let output_dir = cli.output_dir.unwrap_or_else(default_output_dir);

    if !output_dir.exists() {
        eprintln!(
            "Output directory does not exist: {}",
            output_dir.display()
        );
        eprintln!("Run ralph-loop first to create transcripts.");
        std::process::exit(1);
    }

    let discovery = RunDiscovery::new(&output_dir)?;
    let runs = discovery.list_runs()?;

    if runs.is_empty() {
        eprintln!("No runs found in {}", output_dir.display());
        eprintln!("Run ralph-loop first to create transcripts.");
        std::process::exit(1);
    }

    // List mode: just show runs in simple format and exit
    if cli.list {
        println!("Available runs in {}:", output_dir.display());
        println!();
        for run in &runs {
            println!(
                "  {} ({}, {} iteration(s)) - {}",
                run.run_id,
                run.status_display(),
                run.current_iteration(),
                run.prompt_preview
            );
        }
        return Ok(());
    }

    // Summary mode: show detailed summary with duration, tokens, exit reason
    // This is the default when no specific run is requested
    if cli.summary || (cli.run.is_none() && cli.iteration.is_none()) {
        summary::display_summary(&runs);
        return Ok(());
    }

    // Determine which run to view
    let selected_run = if let Some(run_id) = cli.run {
        runs.into_iter()
            .find(|r| r.run_id == run_id || r.run_id.starts_with(&run_id))
            .ok_or_else(|| error::ViewerError::RunNotFound(run_id))?
    } else if runs.len() == 1 {
        // Auto-select single run
        runs.into_iter().next().unwrap()
    } else {
        // Show picker
        picker::select_run(runs)?
    };

    // Determine which iteration to view
    let iteration = if let Some(iter) = cli.iteration {
        Some(iter)
    } else if selected_run.current_iteration() > 1 {
        // Show iteration picker
        picker::select_iteration(&selected_run)?
    } else {
        // Single iteration, auto-select
        Some(1)
    };

    // View the transcript
    if cli.no_follow {
        // Static view
        formatter::display_transcript(&selected_run, iteration)?;
    } else {
        // Live view with file watching
        watcher::watch_transcript(&selected_run, iteration).await?;
    }

    Ok(())
}
