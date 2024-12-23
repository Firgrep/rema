use std::{error::Error, process::Command};

const GIT_MIN_VERSION: &str = "2.43.0";
const GIT_MAX_VERSION: &str = "3.0.0";

pub fn verify_git_version() -> Result<bool, Box<dyn Error>> {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute GitHub CLI: {}", e))?;

    if !output.status.success() {
        return Ok(false);
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    let version = version_str
        .split_whitespace()
        .nth(2)
        .ok_or("Failed to parse Git version")?;

    let min_version = semver::Version::parse(GIT_MIN_VERSION)?;
    let max_version = semver::Version::parse(GIT_MAX_VERSION)?;
    let current_version = semver::Version::parse(version)?;

    if current_version < min_version {
        return Err(format!(
            "Git version is too old: {}. Minimum required version is {}.",
            current_version, min_version
        )
        .into());
    }

    if current_version >= max_version {
        return Err(format!(
            "Git version is too new: {}. Maximum supported version is {}.",
            current_version, max_version
        )
        .into());
    }

    Ok(true)
}

pub fn verify_no_outstanding_commits() -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .expect("Failed to execute git status");

    if !output.stdout.is_empty() {
        return Err(format!(
            "Kindly commit into git any outstanding changes before proceeding. Run 'git status' to see the changes"
        )
        .into());
    } else {
        Ok(())
    }
}
