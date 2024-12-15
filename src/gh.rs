use std::{error::Error, process::Command};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
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
