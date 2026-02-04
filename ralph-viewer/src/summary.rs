//! Summary display for ralph-loop runs.

use colored::Colorize;

use crate::run::{format_tokens, RunMetadata, RunStatus};

/// Display a summary of all runs
pub fn display_summary(runs: &[RunMetadata]) {
    if runs.is_empty() {
        println!("No runs found.");
        return;
    }

    // Separate running and completed runs
    let (running, completed): (Vec<_>, Vec<_>) = runs.iter().partition(|r| r.is_active());

    // Display currently running loops first
    if !running.is_empty() {
        println!("{}", "Currently Running".bold().green());
        println!("{}", "─".repeat(80));
        for run in &running {
            display_run_row(run);
        }
        println!();
    }

    // Display completed runs
    if !completed.is_empty() {
        println!("{}", "Completed Runs".bold());
        println!("{}", "─".repeat(80));
        for run in &completed {
            display_run_row(run);
        }
    }

    // Summary statistics
    println!();
    println!("{}", "─".repeat(80));
    let total_tokens: usize = runs.iter().map(|r| r.total_tokens()).sum();
    let total_runs = runs.len();
    let running_count = running.len();
    println!(
        "{} total runs, {} running, {} tokens used",
        total_runs.to_string().bold(),
        running_count.to_string().bold(),
        format_tokens(total_tokens).bold()
    );
}

/// Display a single run as a row
fn display_run_row(run: &RunMetadata) {
    let status_indicator = match run.status {
        RunStatus::Running => "●".green(),
        RunStatus::Completed => "✓".blue(),
        RunStatus::Failed => "✗".red(),
        RunStatus::Interrupted => "○".yellow(),
    };

    let duration = run.duration_display();
    let tokens = format_tokens(run.total_tokens());
    let iterations = run.current_iteration();
    let exit_reason = run.exit_reason_display();

    // Format: [indicator] run_id | duration | iterations | tokens | exit reason | prompt preview
    println!(
        "{} {} {} {} {} {} {} {} {} {}",
        status_indicator,
        truncate(&run.run_id, 24).dimmed(),
        "|".dimmed(),
        format!("{:>10}", duration).cyan(),
        "|".dimmed(),
        format!("{} iter", iterations),
        "|".dimmed(),
        format!("{:>8} tok", tokens),
        "|".dimmed(),
        format!("{:12}", exit_reason).yellow(),
    );

    // Second line: prompt preview
    println!(
        "  {} {}",
        "→".dimmed(),
        truncate(&run.prompt_preview, 70).dimmed()
    );
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        assert_eq!(truncate("hello world!", 8), "hello...");
    }

    #[test]
    fn test_truncate_exact() {
        assert_eq!(truncate("hello", 5), "hello");
    }
}
