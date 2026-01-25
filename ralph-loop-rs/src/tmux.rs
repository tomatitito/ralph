//! Tmux session management for ralph-loop.
//!
//! This module provides functionality to:
//! - Start ralph-loop in a new tmux session
//! - Launch the viewer in a separate tmux pane/window

use std::path::Path;
use std::process::Command;

use crate::error::{RalphError, Result};

/// Default tmux session name for ralph-loop
pub const DEFAULT_SESSION_NAME: &str = "ralph";

/// Check if tmux is available on the system
pub fn is_tmux_available() -> bool {
    Command::new("tmux")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if we're currently inside a tmux session
pub fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

/// Check if a tmux session exists
pub fn session_exists(session_name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", session_name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Start ralph-loop in a new tmux session
///
/// This spawns a new tmux session with ralph-loop running inside it.
/// Returns immediately after starting the session.
pub fn start_in_tmux_session(
    session_name: &str,
    ralph_loop_args: &[String],
    viewer_path: Option<&Path>,
    output_dir: &Path,
) -> Result<()> {
    if !is_tmux_available() {
        return Err(RalphError::TmuxNotAvailable);
    }

    // Get the path to the current executable
    let current_exe = std::env::current_exe().map_err(|e| RalphError::TmuxError(e.to_string()))?;

    // Build the ralph-loop command
    let mut ralph_cmd = format!("{}", current_exe.display());
    for arg in ralph_loop_args {
        // Quote arguments that contain spaces
        if arg.contains(' ') {
            ralph_cmd.push_str(&format!(" \"{}\"", arg));
        } else {
            ralph_cmd.push_str(&format!(" {}", arg));
        }
    }

    // Kill existing session if it exists
    if session_exists(session_name) {
        let _ = Command::new("tmux")
            .args(["kill-session", "-t", session_name])
            .output();
    }

    // Create new tmux session with ralph-loop
    let mut tmux_cmd = Command::new("tmux");
    tmux_cmd.args([
        "new-session",
        "-d", // detached
        "-s",
        session_name,
        "-n",
        "loop", // window name
    ]);
    tmux_cmd.arg(&ralph_cmd);

    let output = tmux_cmd
        .output()
        .map_err(|e| RalphError::TmuxError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RalphError::TmuxError(format!(
            "Failed to create tmux session: {}",
            stderr
        )));
    }

    // If viewer path is provided, start it in a new window
    if let Some(viewer) = viewer_path {
        let viewer_cmd = format!("{} --dir {}", viewer.display(), output_dir.display());

        let output = Command::new("tmux")
            .args([
                "new-window",
                "-t",
                &format!("{}:", session_name),
                "-n",
                "viewer",
            ])
            .arg(&viewer_cmd)
            .output()
            .map_err(|e| RalphError::TmuxError(e.to_string()))?;

        if !output.status.success() {
            // Non-fatal, just log
            eprintln!(
                "Warning: Failed to start viewer window: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    Ok(())
}

/// Attach to an existing tmux session
pub fn attach_to_session(session_name: &str) -> Result<()> {
    if !session_exists(session_name) {
        return Err(RalphError::TmuxError(format!(
            "Session '{}' does not exist",
            session_name
        )));
    }

    let status = Command::new("tmux")
        .args(["attach-session", "-t", session_name])
        .status()
        .map_err(|e| RalphError::TmuxError(e.to_string()))?;

    if !status.success() {
        return Err(RalphError::TmuxError(
            "Failed to attach to session".to_string(),
        ));
    }

    Ok(())
}

/// Start the viewer in a new tmux window within the current session
pub fn start_viewer_window(viewer_path: &Path, output_dir: &Path) -> Result<()> {
    if !is_inside_tmux() {
        return Err(RalphError::TmuxError(
            "Not inside a tmux session".to_string(),
        ));
    }

    let viewer_cmd = format!("{} --dir {}", viewer_path.display(), output_dir.display());

    let output = Command::new("tmux")
        .args(["new-window", "-n", "viewer"])
        .arg(&viewer_cmd)
        .output()
        .map_err(|e| RalphError::TmuxError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RalphError::TmuxError(format!(
            "Failed to create viewer window: {}",
            stderr
        )));
    }

    Ok(())
}

/// Find the viewer binary
pub fn find_viewer() -> Option<std::path::PathBuf> {
    // First check if ralph-viewer is in the same directory as ralph-loop
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let viewer_path = dir.join("ralph-viewer");
            if viewer_path.exists() {
                return Some(viewer_path);
            }
        }
    }

    // Check if it's in PATH
    if Command::new("ralph-viewer")
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some(std::path::PathBuf::from("ralph-viewer"));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tmux_available_check() {
        // Just ensure it doesn't panic
        let _ = is_tmux_available();
    }

    #[test]
    fn test_is_inside_tmux_check() {
        // Just ensure it doesn't panic
        let _ = is_inside_tmux();
    }

    #[test]
    fn test_session_exists_nonexistent() {
        // A random session name should not exist
        assert!(!session_exists("ralph_test_nonexistent_12345"));
    }
}
