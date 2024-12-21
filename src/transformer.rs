use semver::{BuildMetadata, Prerelease, Version};

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

pub fn extract_pkgs_and_latest_versions(releases: Vec<Release>) -> HashMap<String, Version> {
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

pub enum PreReleaseType {
    Alpha,
    Beta,
    Rc,
}

pub enum BaseVersion {
    Major,
    Minor,
    Patch,
}

pub enum VersionBump {
    Major,
    Minor,
    Patch,
    Pre,
    PreNew(PreReleaseType, BaseVersion),
}

pub fn bump_version(version: &Version, bump: VersionBump) -> Version {
    match bump {
        VersionBump::Major => Version {
            major: version.major + 1,
            minor: version.minor,
            patch: version.patch,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        },
        VersionBump::Minor => Version {
            major: version.major,
            minor: version.minor + 1,
            patch: version.patch,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        },
        VersionBump::Patch => Version {
            major: version.major,
            minor: version.minor,
            patch: version.patch + 1,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        },
        VersionBump::Pre => Version {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            pre: increment_pre(&version.pre),
            build: BuildMetadata::EMPTY,
        },
        VersionBump::PreNew(pre_type, base) => generate_pre_release(version, base, pre_type),
    }
}

fn generate_pre_release(version: &Version, base: BaseVersion, pre_type: PreReleaseType) -> Version {
    let (major, minor, patch) = match base {
        BaseVersion::Major => (version.major + 1, 0, 0),
        BaseVersion::Minor => (version.major, version.minor + 1, 0),
        BaseVersion::Patch => (version.major, version.minor, version.patch + 1),
    };

    let pre_str = match pre_type {
        PreReleaseType::Alpha => "alpha.1",
        PreReleaseType::Beta => "beta.1",
        PreReleaseType::Rc => "rc.1",
    };

    Version {
        major,
        minor,
        patch,
        pre: Prerelease::new(pre_str).unwrap(),
        build: BuildMetadata::EMPTY,
    }
}

fn increment_pre(pre: &Prerelease) -> Prerelease {
    if let Some((ident, num_str)) = pre.as_str().split_once('.') {
        if let Ok(num) = num_str.parse::<u64>() {
            return Prerelease::new(&format!("{}.{}", ident, num + 1)).unwrap();
        }
    }
    panic!(
        "Invalid pre-release format: {}. Expected to delimit by '.'. Example 'beta.1'",
        pre
    );
}
