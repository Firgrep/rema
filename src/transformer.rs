use semver::{BuildMetadata, Prerelease, Version};

use crate::{ctx::AppContext, gh::Release};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum PreReleaseType {
    Alpha,
    Beta,
    Rc,
}

#[derive(Debug, Clone)]
pub enum PreReleaseVersionBump {
    Major,
    Minor,
    Patch,
    Retain,
}

#[derive(Debug, Clone)]
pub enum VersionBump {
    Major,
    Minor,
    Patch,
    Pre,
    PreNew(PreReleaseType, PreReleaseVersionBump),
}

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: Version,
    pub has_v_prefix: bool,
}

pub fn extract_pkgs_and_all_versions(releases: Vec<Release>) -> HashMap<String, Vec<ReleaseInfo>> {
    let mut all_versions: HashMap<String, Vec<ReleaseInfo>> = HashMap::new();

    for release in releases {
        // For monorepos with multiple packages
        if let Some((app_name, version_str)) = release.tag_name.split_once('@') {
            // Remove the 'v' prefix if present
            let version_str = version_str.strip_prefix('v').unwrap_or(version_str);

            if let Ok(version) = Version::parse(version_str) {
                if let Some(versions) = all_versions.get_mut(app_name) {
                    let release_info = ReleaseInfo {
                        version,
                        has_v_prefix: version_str.starts_with('v'),
                    };
                    versions.push(release_info);
                } else {
                    let release_info = ReleaseInfo {
                        version,
                        has_v_prefix: version_str.starts_with('v'),
                    };
                    all_versions.insert(app_name.to_string(), vec![release_info]);
                }
            } else {
                panic!("Invalid version format for tag: {}", release.tag_name);
            }
        } else {
            let app_name = ""; // TODO: get the app name from the repo name, etc.
            let version_str = release
                .tag_name
                .strip_prefix('v')
                .unwrap_or(release.tag_name.as_str());

            if let Ok(version) = Version::parse(version_str) {
                if let Some(versions) = all_versions.get_mut(app_name) {
                    let release_info = ReleaseInfo {
                        version,
                        has_v_prefix: version_str.starts_with('v'),
                    };
                    versions.push(release_info);
                } else {
                    let release_info = ReleaseInfo {
                        version,
                        has_v_prefix: version_str.starts_with('v'),
                    };
                    all_versions.insert(app_name.to_string(), vec![release_info]);
                }
            } else {
                panic!("Invalid version format for tag: {}", release.tag_name);
            }
        }
    }

    all_versions
}

pub fn extract_pkgs_and_latest_versions(releases: Vec<Release>) -> HashMap<String, ReleaseInfo> {
    let mut latest_versions: HashMap<String, ReleaseInfo> = HashMap::new();

    for release in releases {
        if let Some((app_name, version_str)) = release.tag_name.split_once('@') {
            // Remove the 'v' prefix if present
            let version_str = version_str.strip_prefix('v').unwrap_or(version_str);

            if let Ok(version) = Version::parse(version_str) {
                // Check if the current version is newer
                if let Some(current_rel) = latest_versions.get(app_name) {
                    if &version > &current_rel.version {
                        let release_info = ReleaseInfo {
                            version,
                            has_v_prefix: version_str.starts_with('v'),
                        };
                        latest_versions.insert(app_name.to_string(), release_info);
                    }
                } else {
                    // Insert the first version encountered
                    let release_info = ReleaseInfo {
                        version,
                        has_v_prefix: version_str.starts_with('v'),
                    };
                    latest_versions.insert(app_name.to_string(), release_info);
                }
            } else {
                panic!("Invalid version format for tag: {}", release.tag_name);
            }
        }
    }

    latest_versions
}

pub fn bump_version(ctx: &AppContext, bump: VersionBump) -> ReleaseInfo {
    let latest_versions = ctx.get_latest_versions();
    let selected_pkg = ctx.get_selected_package().unwrap_or_else(|| {
        panic!("No package selected");
    });

    let selected_pkg_release_info = latest_versions.get(selected_pkg).unwrap_or_else(|| {
        panic!("Failed to get version for package: {}", selected_pkg);
    });

    let version = selected_pkg_release_info.version.clone();
    let has_v_prefix = selected_pkg_release_info.has_v_prefix.clone();

    match bump {
        VersionBump::Major => ReleaseInfo {
            version: Version {
                major: version.major + 1,
                minor: 0,
                patch: 0,
                pre: Prerelease::EMPTY,
                build: BuildMetadata::EMPTY,
            },
            has_v_prefix: has_v_prefix,
        },
        VersionBump::Minor => ReleaseInfo {
            version: Version {
                major: version.major,
                minor: version.minor + 1,
                patch: 0,
                pre: Prerelease::EMPTY,
                build: BuildMetadata::EMPTY,
            },
            has_v_prefix: has_v_prefix,
        },
        VersionBump::Patch => ReleaseInfo {
            version: Version {
                major: version.major,
                minor: version.minor,
                patch: version.patch + 1,
                pre: Prerelease::EMPTY,
                build: BuildMetadata::EMPTY,
            },
            has_v_prefix: has_v_prefix,
        },
        VersionBump::Pre => ReleaseInfo {
            version: Version {
                major: version.major,
                minor: version.minor,
                patch: version.patch,
                pre: increment_pre(&version.pre),
                build: BuildMetadata::EMPTY,
            },
            has_v_prefix: has_v_prefix,
        },
        VersionBump::PreNew(pre_type, base) => generate_pre_release(ctx, &version, base, pre_type),
    }
}

fn generate_pre_release(
    ctx: &AppContext,
    existing_version: &Version,
    base: PreReleaseVersionBump,
    pre_type: PreReleaseType,
) -> ReleaseInfo {
    let (major, minor, patch) = match base {
        PreReleaseVersionBump::Major => (existing_version.major + 1, 0, 0),
        PreReleaseVersionBump::Minor => (existing_version.major, existing_version.minor + 1, 0),
        PreReleaseVersionBump::Patch => (
            existing_version.major,
            existing_version.minor,
            existing_version.patch + 1,
        ),
        PreReleaseVersionBump::Retain => (
            existing_version.major,
            existing_version.minor,
            existing_version.patch,
        ),
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
    let latest_versions = ctx.get_latest_versions();
    let selected_pkg_release_info = latest_versions.get(pkg_name).unwrap_or_else(|| {
        panic!("Failed to get version for package: {}", pkg_name);
    });
    let has_v_prefix = selected_pkg_release_info.has_v_prefix.clone();

    let new_release_info = ReleaseInfo {
        version: Version {
            major,
            minor,
            patch,
            pre: Prerelease::new(pre_str).unwrap(),
            build: BuildMetadata::EMPTY,
        },
        has_v_prefix,
    };

    // Check if the pre-release already exists.
    let existing_pre = ctx.find_existing_prerelease(pkg_name, &new_release_info.version, pre_type);
    if existing_pre.is_some() {
        let existing_pre = existing_pre.unwrap();
        panic!(
            "Failed to generate pre-release. {} already exists, or is of older version, for {} ({}.{}.{}-{})",
            new_release_info.version,
            pkg_name,
            existing_pre.version.major,
            existing_pre.version.minor,
            existing_pre.version.patch,
            existing_pre.version.pre
        )
    }

    new_release_info
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
                tag_name: "tiger@v1.1.0".to_string(),
                ..Default::default()
            },
        ];

        let ctx = AppContext::new(test_releases);
        let app_names = ctx.get_pkgs();
        assert_eq!(app_names.len(), 2);
        assert!(app_names.contains(&"tiger".to_string()));
        assert!(app_names.contains(&"elephant".to_string()));
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
            latest_versions.get("tiger").unwrap().version,
            Version::new(1, 1, 0)
        );
        assert_eq!(
            latest_versions.get("elephant").unwrap().version,
            Version::new(1, 0, 0)
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
            latest_versions.get("tiger").unwrap().version,
            Version::new(1, 1, 0)
        );

        let mut test_release = Version::new(1, 0, 1);
        test_release.pre = Prerelease::new("rc.2").unwrap();

        assert_eq!(
            latest_versions.get("elephant").unwrap().version,
            test_release
        );
    }

    #[test]
    fn should_panic_when_generating_exact_same_existing_pre_release() {
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
        ctx.set_selected_package("elephant".to_string());

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
    fn should_panic_when_same_pre_release_type_if_newer_exist() {
        // Should not be able to create a new pre-release of the
        // same type if it already exists on version
        // e.g.
        // when v1.0.0-rc.3 exists,
        // attempting to make v1.0.0-rc.1 should fail.

        // same for other pre-release types. They should not be visible on the CLI
        let test_releases = vec![
            Release {
                tag_name: "elephant@v1.0.1-rc.3".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "tiger@v1.1.0".to_string(),
                ..Default::default()
            },
        ];

        let mut ctx = AppContext::new(test_releases);
        ctx.set_selected_package("elephant".to_string());

        // Selected version (CLI)
        let existing_version = Version::new(1, 0, 0);
        let base = PreReleaseVersionBump::Patch;
        let pre_type = PreReleaseType::Rc;

        let mut ctrl_version = Version::new(1, 0, 2);
        ctrl_version.pre = Prerelease::new("rc.1").unwrap();

        let result = std::panic::catch_unwind(|| {
            generate_pre_release(&ctx, &existing_version, base, pre_type)
        });
        assert!(result.is_err());
    }
}
