//! Interactive run and iteration picker UI.

use inquire::Select;

use crate::error::{Result, ViewerError};
use crate::run::RunMetadata;

/// Select a run from a list of available runs
pub fn select_run(runs: Vec<RunMetadata>) -> Result<RunMetadata> {
    if runs.is_empty() {
        return Err(ViewerError::NoRunsAvailable);
    }

    let options: Vec<String> = runs
        .iter()
        .map(|r| {
            format!(
                "{} ({}, {} iter) - {}",
                r.run_id,
                r.status_display(),
                r.current_iteration(),
                truncate(&r.prompt_preview, 40)
            )
        })
        .collect();

    let selection = Select::new("Select a run:", options)
        .with_help_message("↑↓ to move, Enter to select, Esc to cancel")
        .prompt()
        .map_err(|_| ViewerError::UserCancelled)?;

    // Find the matching run by the selection string
    let index = runs
        .iter()
        .position(|r| {
            selection.starts_with(&r.run_id)
        })
        .ok_or(ViewerError::UserCancelled)?;

    Ok(runs.into_iter().nth(index).unwrap())
}

/// Select an iteration from a run
pub fn select_iteration(run: &RunMetadata) -> Result<Option<u32>> {
    let current = run.current_iteration();
    if current == 0 {
        return Ok(None);
    }

    let mut options = Vec::new();

    // Add "current (live)" option if run is active
    if run.is_active() {
        options.push(format!(
            "current (iteration {}, live)",
            current
        ));
    }

    // Add individual iterations
    for i in (1..=current).rev() {
        if run.is_active() && i == current {
            continue; // Skip, already added as "current"
        }
        options.push(format!("iteration {}", i));
    }

    // Add "all" option
    options.push("[all] View full transcript".to_string());

    let selection = Select::new("Select iteration:", options)
        .with_help_message("↑↓ to move, Enter to select, Esc to cancel")
        .prompt()
        .map_err(|_| ViewerError::UserCancelled)?;

    if selection.starts_with("[all]") {
        Ok(None) // None means "all iterations"
    } else if selection.starts_with("current") {
        Ok(Some(run.current_iteration()))
    } else if selection.starts_with("iteration ") {
        let iter_str = selection.strip_prefix("iteration ").unwrap();
        let iter: u32 = iter_str.parse().map_err(|_| ViewerError::UserCancelled)?;
        Ok(Some(iter))
    } else {
        Err(ViewerError::UserCancelled)
    }
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
}
