use std::{error::Error, process::Command, str};

use semver::Version;

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

/// Create a release commit with the target version as message.
///
/// Will handle and verify staging.
pub fn create_release_commit(version: &Version) -> Result<(), Box<dyn Error>> {
    // Stage all changes using -A flag
    let stage_result = Command::new("git")
        .args(["add", "-A"])
        .output()
        .map_err(|e| format!("Failed to stage changes: {}", e))?;

    if !stage_result.status.success() {
        let stderr = str::from_utf8(&stage_result.stderr)
            .map_err(|e| format!("Failed to parse git staging error: {}", e))?;
        return Err(format!("Failed to stage changes: {}", stderr).into());
    }

    // Verify changes were staged
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("Failed to check git status: {}", e))?;

    let status_str = str::from_utf8(&status.stdout)
        .map_err(|e| format!("Failed to parse git status output: {}", e))?;

    if status_str.trim().is_empty() {
        return Err("No changes staged for commit".into());
    }

    // Run commit
    let commit_result = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(format!("Release version {}", version))
        .output()
        .map_err(|e| format!("Failed to execute git commit: {}", e))?;

    if !commit_result.status.success() {
        let stderr = str::from_utf8(&commit_result.stderr)
            .map_err(|e| format!("Failed to parse git error output: {}", e))?;
        let stdout = str::from_utf8(&commit_result.stdout)
            .map_err(|e| format!("Failed to parse git output: {}", e))?;

        // Combining stdout and stderr for more complete error information
        let error_msg = if stderr.trim().is_empty() {
            if stdout.trim().is_empty() {
                "Git commit failed with no error message".to_string()
            } else {
                format!("Git commit failed: {}", stdout.trim())
            }
        } else {
            format!("Git commit failed: {}", stderr.trim())
        };

        return Err(error_msg.into());
    }

    Ok(())
}

pub fn push() -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("push")
        .output()
        .expect("Failed to execute git push");

    if !output.status.success() {
        let stderr = str::from_utf8(&output.stderr)?;
        return Err(format!("Git push failed: {}", stderr).into());
    }

    Ok(())
}

pub fn fetch_tags() -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("fetch")
        .arg("--tags")
        .output()
        .expect("Failed to execute git fetch");

    if !output.status.success() {
        let stderr = str::from_utf8(&output.stderr)?;
        return Err(format!("Git fetch failed: {}", stderr).into());
    }

    Ok(())
}
