use std::{error::Error, process::Command, str};

use serde::{Deserialize, Serialize};

use crate::transform::ReleaseInfo;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Release {
    pub name: String,

    #[serde(rename = "tagName")]
    pub tag_name: String,

    #[serde(rename = "publishedAt")]
    pub published_at: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "isDraft")]
    pub is_draft: bool,

    #[serde(rename = "isPrerelease")]
    pub is_prerelease: bool,

    #[serde(rename = "isLatest")]
    pub is_latest: bool,
}

const GH_CLI_MIN_VERSION: &str = "2.45.0";
const GH_CLI_MAX_VERSION: &str = "3.0.0";

impl Default for Release {
    fn default() -> Self {
        Release {
            name: String::new(),
            tag_name: String::new(),
            published_at: String::new(),
            created_at: String::new(),
            is_draft: false,
            is_prerelease: false,
            is_latest: false,
        }
    }
}

pub fn verify_gh_cli_version() -> Result<bool, Box<dyn Error>> {
    let output = Command::new("gh")
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
        .ok_or("Failed to parse GitHub CLI version")?;

    let min_version = semver::Version::parse(GH_CLI_MIN_VERSION)?;
    let max_version = semver::Version::parse(GH_CLI_MAX_VERSION)?;
    let current_version = semver::Version::parse(version)?;

    if current_version < min_version {
        return Err(format!(
            "GitHub CLI version is too old: {}. Minimum required version is {}.",
            current_version, min_version
        )
        .into());
    }

    if current_version >= max_version {
        return Err(format!(
            "GitHub CLI version is too new: {}. Maximum supported version is {}.",
            current_version, max_version
        )
        .into());
    }

    Ok(true)
}

pub fn list_releases() -> Result<Vec<Release>, Box<dyn Error>> {
    let output = Command::new("gh")
        .args(&[
            "release",
            "list",
            "--json",
            "createdAt,isDraft,isLatest,isPrerelease,name,publishedAt,tagName",
        ])
        .output()
        .map_err(|e| format!("Failed to execute GitHub CLI: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "GitHub CLI returned non-success status: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let releases: Vec<Release> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse releases: {}", e))?;

    Ok(releases)
}

pub fn create_release(
    release_info: ReleaseInfo,
    target_description: String,
    target_title: String,
) -> Result<(), Box<dyn Error>> {
    let version = release_info.version.to_string();
    let notes = format!("{:?}", target_description);

    let mut command_args = vec![
        "release",
        "create",
        &version,
        "--notes",
        &notes,
        "--title",
        &target_title,
        "--generate-notes",
        "--verify-tag",
    ];

    println!("{:?}", command_args);
    panic!();
    if !release_info.version.pre.is_empty() {
        command_args.push("--prerelease");
    }

    let output = Command::new("gh")
        .args(&command_args)
        .output()
        .expect("Failed to execute gh release create");

    if !output.status.success() {
        let stderr = str::from_utf8(&output.stderr)?;
        return Err(format!("Git push failed: {}", stderr).into());
    }

    Ok(())
}
