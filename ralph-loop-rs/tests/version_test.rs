//! Integration test to verify the --version flag shows the correct version from Cargo.toml

use std::process::Command;

#[test]
fn version_flag_shows_cargo_version() {
    // Get the version from Cargo.toml
    let cargo_version = env!("CARGO_PKG_VERSION");

    // Run the binary with --version
    let output = Command::new(env!("CARGO_BIN_EXE_ralph-loop"))
        .arg("--version")
        .output()
        .expect("Failed to execute ralph-loop --version");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // The output should be "ralph-loop <version>"
    assert!(
        output.status.success(),
        "ralph-loop --version should exit successfully"
    );
    assert!(
        stdout.contains(cargo_version),
        "Output '{}' should contain version '{}'",
        stdout.trim(),
        cargo_version
    );
    assert!(
        stdout.contains("ralph-loop"),
        "Output '{}' should contain 'ralph-loop'",
        stdout.trim()
    );
    assert_eq!(stdout.trim(), format!("ralph-loop {}", cargo_version));
}

#[test]
fn short_version_flag_shows_cargo_version() {
    let cargo_version = env!("CARGO_PKG_VERSION");

    let output = Command::new(env!("CARGO_BIN_EXE_ralph-loop"))
        .arg("-V")
        .output()
        .expect("Failed to execute ralph-loop -V");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "ralph-loop -V should exit successfully"
    );
    assert!(
        stdout.contains(cargo_version),
        "Output '{}' should contain version '{}'",
        stdout.trim(),
        cargo_version
    );
    assert_eq!(stdout.trim(), format!("ralph-loop {}", cargo_version));
}

#[test]
fn upgrade_help_is_available_without_prompt() {
    let output = Command::new(env!("CARGO_BIN_EXE_ralph-loop"))
        .args(["upgrade", "--help"])
        .output()
        .expect("Failed to execute ralph-loop upgrade --help");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "ralph-loop upgrade --help should exit successfully"
    );
    assert!(
        stdout.contains("Upgrade ralph-loop to the latest GitHub release"),
        "Output '{}' should describe the upgrade command",
        stdout.trim()
    );
}
