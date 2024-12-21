use semver::{BuildMetadata, Prerelease, Version};

use crate::{ctx::AppContext, gh::Release};
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

pub fn extract_pkgs_and_all_versions(releases: Vec<Release>) -> HashMap<String, Vec<Version>> {
    let mut all_versions: HashMap<String, Vec<Version>> = HashMap::new();

    for release in releases {
        if let Some((app_name, version_str)) = release.tag_name.split_once('@') {
            // Remove the 'v' prefix if present
            let version_str = version_str.strip_prefix('v').unwrap_or(version_str);

            if let Ok(version) = Version::parse(version_str) {
                if let Some(versions) = all_versions.get_mut(app_name) {
                    versions.push(version);
                } else {
                    all_versions.insert(app_name.to_string(), vec![version]);
                }
            } else {
                panic!("Invalid version format for tag: {}", release.tag_name);
            }
        }
    }

    all_versions
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

pub enum PreReleaseVersionBump {
    Major,
    Minor,
    Patch,
    Retain,
}

pub enum VersionBump {
    Major,
    Minor,
    Patch,
    Pre,
    PreNew(PreReleaseType, PreReleaseVersionBump),
}

pub fn bump_version(ctx: &AppContext, version: &Version, bump: VersionBump) -> Version {
    match bump {
        VersionBump::Major => Version {
            major: version.major + 1,
            minor: 0,
            patch: 0,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        },
        VersionBump::Minor => Version {
            major: version.major,
            minor: version.minor + 1,
            patch: 0,
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
        VersionBump::PreNew(pre_type, base) => generate_pre_release(ctx, version, base, pre_type),
    }
}

fn generate_pre_release(
    ctx: &AppContext,
    version: &Version,
    base: PreReleaseVersionBump,
    pre_type: PreReleaseType,
) -> Version {
    let (major, minor, patch) = match base {
        PreReleaseVersionBump::Major => (version.major + 1, 0, 0),
        PreReleaseVersionBump::Minor => (version.major, version.minor + 1, 0),
        PreReleaseVersionBump::Patch => (version.major, version.minor, version.patch + 1),
        PreReleaseVersionBump::Retain => (version.major, version.minor, version.patch),
    };

    let pre_str = match pre_type {
        PreReleaseType::Alpha => "alpha.1",
        PreReleaseType::Beta => "beta.1",
        PreReleaseType::Rc => "rc.1",
    };

    let pkg_name = match ctx.get_selected_package() {
        Some(name) => name,
        None => panic!("No package selected"),
    };

    let new_version = Version {
        major,
        minor,
        patch,
        pre: Prerelease::new(pre_str).unwrap(),
        build: BuildMetadata::EMPTY,
    };

    // Check if the pre-release already exists.
    if ctx
        .find_existing_prerelease(pkg_name, &new_version, pre_type)
        .is_some()
    {
        panic!(
            "Failed to generate. Pre-release already exists for package: {} with version: {}",
            pkg_name, new_version
        )
    }

    new_version
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_unique_app_names() {
        let releases = vec![
            Release {
                tag_name: "tiger@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "tiger@v1.1.0".to_string(),
                ..Default::default()
            },
        ];

        let app_names = extract_unique_app_names(releases);
        assert_eq!(app_names.len(), 2);
        assert!(app_names.contains("tiger"));
        assert!(app_names.contains("elephant"));
    }

    #[test]
    fn test_extract_pkgs_and_all_versions() {
        let releases = vec![
            Release {
                tag_name: "tiger@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "tiger@v1.1.0".to_string(),
                ..Default::default()
            },
        ];

        let all_versions = extract_pkgs_and_all_versions(releases);
        assert_eq!(all_versions.len(), 2);
        assert_eq!(all_versions.get("tiger").unwrap().len(), 2);
        assert_eq!(all_versions.get("elephant").unwrap().len(), 1);
    }

    #[test]
    fn test_extract_pkgs_and_latest_versions() {
        let releases = vec![
            Release {
                tag_name: "tiger@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.0-alpha.1".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "tiger@v1.1.0".to_string(),
                ..Default::default()
            },
        ];

        let latest_versions = extract_pkgs_and_latest_versions(releases);
        assert_eq!(latest_versions.len(), 2);
        assert_eq!(
            latest_versions.get("tiger").unwrap(),
            &Version::new(1, 1, 0)
        );
        assert_eq!(
            latest_versions.get("elephant").unwrap(),
            &Version::new(1, 0, 0)
        );
    }

    #[test]
    fn test_extract_pkgs_and_latest_versions_with_pre_release() {
        let releases = vec![
            Release {
                tag_name: "tiger@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.1-alpha.1".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.1-beta.1".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.1-rc.1".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.1-rc.2".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "tiger@v1.1.0".to_string(),
                ..Default::default()
            },
        ];

        let latest_versions = extract_pkgs_and_latest_versions(releases);
        assert_eq!(latest_versions.len(), 2);
        assert_eq!(
            latest_versions.get("tiger").unwrap(),
            &Version::new(1, 1, 0)
        );

        let mut test_release = Version::new(1, 0, 1);
        test_release.pre = Prerelease::new("rc.2").unwrap();

        assert_eq!(latest_versions.get("elephant").unwrap(), &test_release);
    }

    #[test]
    fn generate_exact_same_existing_pre_release() {
        let test_releases = vec![
            Release {
                tag_name: "tiger@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.1-alpha.1".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.1-beta.1".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.1-rc.1".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "tiger@v1.1.0".to_string(),
                ..Default::default()
            },
        ];

        let mut ctx = AppContext::new(test_releases);
        ctx.select_package("elephant".to_string());

        let version = Version::new(1, 0, 0);
        let base = PreReleaseVersionBump::Patch;
        let pre_type = PreReleaseType::Rc;

        let mut ctrl_version = Version::new(1, 0, 2);
        ctrl_version.pre = Prerelease::new("rc.1").unwrap();

        let result =
            std::panic::catch_unwind(|| generate_pre_release(&ctx, &version, base, pre_type));
        assert!(result.is_err());
    }

    #[test]
    fn generate_same_pre_release_type() {
        // TODO
        // should not be able to create a new pre-release of the same type if it already exists on version
        // e.g.
        // v1.0.0-rc.3
        // attempting to make v1.0.0-rc.1 should fail

        // same for other pre-release types. They should not be visible on the CLI
    }
}
