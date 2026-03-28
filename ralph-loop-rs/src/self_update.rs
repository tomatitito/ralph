use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::RalphError;

const GITHUB_REPO: &str = "tomatitito/ralph";
const BINARY_NAME: &str = "ralph-loop";

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub fn upgrade_current_binary() -> Result<String, RalphError> {
    let current_exe = std::env::current_exe().map_err(io_upgrade_error)?;
    let install_dir = current_exe.parent().ok_or_else(|| {
        RalphError::UpgradeError("could not determine current executable directory".to_string())
    })?;
    let platform_suffix = platform_suffix()?;
    let artifact_name = format!("{BINARY_NAME}-{platform_suffix}.tar.gz");

    let release = fetch_latest_release()?;
    let release_version = normalize_version(&release.tag_name);

    if release_version == crate::VERSION {
        return Ok(format!(
            "{BINARY_NAME} {release_version} is already installed"
        ));
    }

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == artifact_name)
        .ok_or_else(|| {
            RalphError::UpgradeError(format!(
                "could not find release artifact '{artifact_name}' in latest release"
            ))
        })?;

    let temp_dir = make_temp_dir()?;
    let archive_path = temp_dir.join(&artifact_name);

    download_file(&asset.browser_download_url, &archive_path)?;
    extract_archive(&archive_path, &temp_dir)?;

    let extracted_binary = temp_dir.join(BINARY_NAME);
    if !extracted_binary.exists() {
        return Err(RalphError::UpgradeError(format!(
            "downloaded archive did not contain '{BINARY_NAME}'"
        )));
    }

    install_binary(&extracted_binary, install_dir, &current_exe)?;

    let _ = fs::remove_dir_all(&temp_dir);

    Ok(format!(
        "upgraded {BINARY_NAME} from {} to {}",
        crate::VERSION,
        release_version
    ))
}

fn platform_suffix() -> Result<&'static str, RalphError> {
    platform_suffix_for(std::env::consts::OS, std::env::consts::ARCH)
}

fn platform_suffix_for(os: &str, arch: &str) -> Result<&'static str, RalphError> {
    match (os, arch) {
        ("linux", "x86_64") => Ok("linux-x86_64"),
        ("macos", "aarch64") => Ok("macos-arm64"),
        (os, arch) => Err(RalphError::UpgradeError(format!(
            "unsupported platform for upgrade: {os}/{arch}"
        ))),
    }
}

fn fetch_latest_release() -> Result<GitHubRelease, RalphError> {
    let output = Command::new("curl")
        .args([
            "-fsSL",
            &format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest"),
        ])
        .output()
        .map_err(io_upgrade_error)?;

    if !output.status.success() {
        return Err(RalphError::UpgradeError(format!(
            "failed to fetch latest release metadata (exit {})",
            output.status
        )));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|err| RalphError::UpgradeError(format!("invalid release metadata: {err}")))
}

fn download_file(url: &str, destination: &Path) -> Result<(), RalphError> {
    let status = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(destination)
        .arg(url)
        .status()
        .map_err(io_upgrade_error)?;

    if !status.success() {
        return Err(RalphError::UpgradeError(format!(
            "failed to download release artifact from {url}"
        )));
    }

    Ok(())
}

fn extract_archive(archive_path: &Path, destination_dir: &Path) -> Result<(), RalphError> {
    let status = Command::new("tar")
        .args(["-xzf"])
        .arg(archive_path)
        .args(["-C"])
        .arg(destination_dir)
        .status()
        .map_err(io_upgrade_error)?;

    if !status.success() {
        return Err(RalphError::UpgradeError(format!(
            "failed to extract archive '{}'",
            archive_path.display()
        )));
    }

    Ok(())
}

fn install_binary(
    extracted_binary: &Path,
    install_dir: &Path,
    current_exe: &Path,
) -> Result<(), RalphError> {
    let staged_path = install_dir.join(format!("{BINARY_NAME}.tmp-{}", Uuid::new_v4()));

    fs::copy(extracted_binary, &staged_path).map_err(io_upgrade_error)?;

    let permissions = fs::metadata(extracted_binary)
        .map_err(io_upgrade_error)?
        .permissions();
    fs::set_permissions(&staged_path, permissions).map_err(io_upgrade_error)?;

    fs::rename(&staged_path, current_exe).map_err(io_upgrade_error)?;

    Ok(())
}

fn make_temp_dir() -> Result<PathBuf, RalphError> {
    let temp_dir = std::env::temp_dir().join(format!("ralph-loop-upgrade-{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).map_err(io_upgrade_error)?;
    Ok(temp_dir)
}

fn normalize_version(version: &str) -> &str {
    version.strip_prefix('v').unwrap_or(version)
}

fn io_upgrade_error(err: std::io::Error) -> RalphError {
    RalphError::UpgradeError(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{normalize_version, platform_suffix_for};

    #[test]
    fn strips_v_prefix_from_release_tag() {
        assert_eq!(normalize_version("v0.4.0"), "0.4.0");
    }

    #[test]
    fn leaves_plain_versions_unchanged() {
        assert_eq!(normalize_version("0.4.0"), "0.4.0");
    }

    #[test]
    fn maps_apple_silicon_macos_to_release_artifact_suffix() {
        assert_eq!(
            platform_suffix_for("macos", "aarch64").unwrap(),
            "macos-arm64"
        );
    }

    #[test]
    fn maps_linux_x86_64_to_release_artifact_suffix() {
        assert_eq!(
            platform_suffix_for("linux", "x86_64").unwrap(),
            "linux-x86_64"
        );
    }

    #[test]
    fn rejects_unsupported_platforms() {
        let error = platform_suffix_for("macos", "x86_64").unwrap_err();
        assert!(error.to_string().contains("unsupported platform"));
    }
}
