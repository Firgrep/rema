use semver::Version;

use crate::gh::Release;
use std::collections::{HashMap, HashSet};

pub fn extract_unique_app_names(releases: Vec<Release>) -> HashSet<String> {
    let mut app_names = HashSet::new();

    for release in releases {
        if let Some((app_name, _)) = release.tag_name.split_once('@') {
            app_names.insert(app_name.to_string());
        }
    }

    app_names
}

pub fn get_latest_versions(releases: Vec<Release>) -> HashMap<String, Version> {
    let mut latest_versions: HashMap<String, Version> = HashMap::new();

    for release in releases {
        if let Some((app_name, version_str)) = release.tag_name.split_once('@') {
            // Remove the 'v' prefix if present
            let version_str = version_str.strip_prefix('v').unwrap_or(version_str);

            if let Ok(version) = Version::parse(version_str) {
                // Check if the current version is newer
                if let Some(current_version) = latest_versions.get(app_name) {
                    if &version > current_version {
                        latest_versions.insert(app_name.to_string(), version);
                    }
                } else {
                    // Insert the first version encountered
                    latest_versions.insert(app_name.to_string(), version);
                }
            } else {
                panic!("Invalid version format for tag: {}", release.tag_name);
            }
        }
    }

    latest_versions
}
