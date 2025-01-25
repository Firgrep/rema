use semver::{BuildMetadata, Prerelease, Version};

use crate::{ctx::AppContext, gh::Release, read::LocalPackageFiles};
use std::{collections::HashMap, error::Error};

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
    RetainIfUnreleased,
    PreNew(PreReleaseType, PreReleaseVersionBump),
}

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: Version,
    pub has_v_prefix: bool,
    pub local_pkg_files: Option<LocalPackageFiles>,
    pub local_only: bool,
}

pub fn extract_all_gh_pkgs_and_versions(
    releases: Vec<Release>,
) -> HashMap<String, Vec<ReleaseInfo>> {
    let mut all_release_info: HashMap<String, Vec<ReleaseInfo>> = HashMap::new();

    for release in releases {
        // For monorepos with multiple packages
        if let Some((app_name, version_str)) = release.tag_name.split_once('@') {
            // Remove the 'v' prefix if present
            let has_started_with_v = version_str.starts_with('v');
            let version_str = version_str.strip_prefix('v').unwrap_or(version_str);

            if let Ok(version) = Version::parse(version_str) {
                if let Some(release_infos) = all_release_info.get_mut(app_name) {
                    let release_info = ReleaseInfo {
                        version,
                        has_v_prefix: has_started_with_v,
                        local_pkg_files: None,
                        local_only: false,
                    };
                    release_infos.push(release_info);
                } else {
                    let release_info = ReleaseInfo {
                        version,
                        has_v_prefix: has_started_with_v,
                        local_pkg_files: None,
                        local_only: false,
                    };
                    all_release_info.insert(app_name.to_string(), vec![release_info]);
                }
            } else {
                panic!("Invalid version format for tag: {}", release.tag_name);
            }
        } else {
            let app_name = ""; // TODO: get the app name from the repo name, etc.
            let has_started_with_v = release.tag_name.starts_with('v');
            let version_str = release
                .tag_name
                .strip_prefix('v')
                .unwrap_or(release.tag_name.as_str());

            if let Ok(version) = Version::parse(version_str) {
                if let Some(versions) = all_release_info.get_mut(app_name) {
                    let release_info = ReleaseInfo {
                        version,
                        has_v_prefix: has_started_with_v,
                        local_pkg_files: None,
                        local_only: false,
                    };
                    versions.push(release_info);
                } else {
                    let release_info = ReleaseInfo {
                        version,
                        has_v_prefix: has_started_with_v,
                        local_pkg_files: None,
                        local_only: false,
                    };
                    all_release_info.insert(app_name.to_string(), vec![release_info]);
                }
            } else {
                panic!("Invalid version format for tag: {}", release.tag_name);
            }
        }
    }

    all_release_info
}

pub fn extract_latest_gh_pkgs_and_versions(
    all_releases: &HashMap<String, Vec<ReleaseInfo>>,
) -> HashMap<String, ReleaseInfo> {
    let mut latest_versions: HashMap<String, ReleaseInfo> = HashMap::new();

    for (app_name, releases) in all_releases {
        for release in releases {
            // Check if the current version is newer
            if let Some(current_rel) = latest_versions.get(app_name) {
                if &release.version > &current_rel.version {
                    let release_info = ReleaseInfo {
                        version: release.version.clone(),
                        has_v_prefix: release.has_v_prefix.clone(),
                        local_pkg_files: release.local_pkg_files.clone(),
                        local_only: release.local_only.clone(),
                    };
                    latest_versions.insert(app_name.to_string(), release_info);
                }
            } else {
                // Insert the first version encountered
                let release_info = ReleaseInfo {
                    version: release.version.clone(),
                    has_v_prefix: release.has_v_prefix.clone(),
                    local_pkg_files: release.local_pkg_files.clone(),
                    local_only: release.local_only.clone(),
                };
                latest_versions.insert(app_name.to_string(), release_info);
            }
        }
    }

    latest_versions
}

pub fn create_release_title(ctx: &AppContext) -> String {
    let release_info = ctx.get_target_release_info().unwrap_or_else(|| {
        panic!("No target release info found. Run `bump` command first");
    });
    let app_name = ctx.get_selected_package().unwrap_or_else(|| {
        panic!("No package selected");
    });
    let version = release_info.version.to_string();

    if app_name.is_empty() {
        if release_info.has_v_prefix {
            format!("v{}", version)
        } else {
            version
        }
    } else {
        if release_info.has_v_prefix {
            format!("{}@v{}", app_name, version)
        } else {
            format!("{}@{}", app_name, version)
        }
    }
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
    let local_pkg_files = selected_pkg_release_info.local_pkg_files.clone();
    let local_only = selected_pkg_release_info.local_only.clone();

    match bump {
        VersionBump::RetainIfUnreleased => ReleaseInfo {
            version: Version {
                major: version.major,
                minor: version.minor,
                patch: version.patch,
                pre: version.pre.clone(),
                build: BuildMetadata::EMPTY,
            },
            has_v_prefix,
            local_pkg_files,
            local_only,
        },
        VersionBump::Major => ReleaseInfo {
            version: Version {
                major: version.major + 1,
                minor: 0,
                patch: 0,
                pre: Prerelease::EMPTY,
                build: BuildMetadata::EMPTY,
            },
            has_v_prefix,
            local_pkg_files,
            local_only,
        },
        VersionBump::Minor => ReleaseInfo {
            version: Version {
                major: version.major,
                minor: version.minor + 1,
                patch: 0,
                pre: Prerelease::EMPTY,
                build: BuildMetadata::EMPTY,
            },
            has_v_prefix,
            local_pkg_files,
            local_only,
        },
        VersionBump::Patch => ReleaseInfo {
            version: Version {
                major: version.major,
                minor: version.minor,
                patch: version.patch + 1,
                pre: Prerelease::EMPTY,
                build: BuildMetadata::EMPTY,
            },
            has_v_prefix,
            local_pkg_files,
            local_only,
        },
        VersionBump::Pre => ReleaseInfo {
            version: Version {
                major: version.major,
                minor: version.minor,
                patch: version.patch,
                pre: increment_pre(&version.pre),
                build: BuildMetadata::EMPTY,
            },
            has_v_prefix,
            local_pkg_files,
            local_only,
        },
        VersionBump::PreNew(pre_type, base) => generate_pre_release(ctx, &version, base, pre_type),
    }
}

pub fn match_local_pkgs_with_gh_pkgs(
    gh_versions: &mut HashMap<String, ReleaseInfo>,
    local_pkg_files: &[LocalPackageFiles],
) -> Result<HashMap<String, ReleaseInfo>, Box<dyn Error>> {
    // Create a clone of the HashMap to modify and return
    let mut updated_versions = gh_versions.clone();

    // Keep track of which local packages that have been matched
    let mut matched_local_pkgs = Vec::new();

    // First, match GitHub versions with local packages
    for (gh_pkg_name, release_info) in updated_versions.iter_mut() {
        if let Some(matching_pkg) = local_pkg_files.iter().find(|local_pkg| {
            local_pkg
                .name
                .as_ref()
                .map_or(false, |name| name == gh_pkg_name)
        }) {
            release_info.local_pkg_files = Some(matching_pkg.clone());
            matched_local_pkgs.push(matching_pkg.name.clone().unwrap_or_default());
        }
    }

    // Now handle unmatched local packages
    for local_pkg in local_pkg_files {
        // Skip already matched this package
        if let Some(pkg_name) = &local_pkg.name {
            if matched_local_pkgs.contains(pkg_name) {
                continue;
            }

            let existing_version = local_pkg
                .package_json
                .as_ref()
                .and_then(|pkg| pkg.version.as_ref())
                .map_or(Version::new(0, 0, 0), |v| Version::parse(v).unwrap());

            // Create a new ReleaseInfo for unmatched local package
            let local_only_release = ReleaseInfo {
                version: existing_version,
                has_v_prefix: false,
                local_pkg_files: Some(local_pkg.clone()),
                local_only: true,
            };

            updated_versions.insert(pkg_name.clone(), local_only_release);
        }
    }

    Ok(updated_versions)
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
    let local_pkg_files = selected_pkg_release_info.local_pkg_files.clone();
    let local_only = selected_pkg_release_info.local_only.clone();

    let new_release_info = ReleaseInfo {
        version: Version {
            major,
            minor,
            patch,
            pre: Prerelease::new(pre_str).unwrap(),
            build: BuildMetadata::EMPTY,
        },
        has_v_prefix,
        local_pkg_files,
        local_only,
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
    fn attach_v_prefix_to_title() {
        let r = vec![
            Release {
                tag_name: "tiger@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@1.0.0".to_string(),
                ..Default::default()
            },
        ];

        let r_t = ReleaseInfo {
            version: Version::new(1, 0, 1),
            has_v_prefix: true,
            local_pkg_files: None,
            local_only: false,
        };
        let r_e = ReleaseInfo {
            version: Version::new(1, 0, 1),
            has_v_prefix: false,
            local_pkg_files: None,
            local_only: false,
        };

        let mut ctx = AppContext::new(r);

        ctx.set_selected_package("tiger".to_string());
        ctx.set_target_release_info(r_t);
        assert_eq!(create_release_title(&ctx), "tiger@v1.0.1");

        ctx.set_selected_package("elephant".to_string());
        ctx.set_target_release_info(r_e);
        assert_eq!(create_release_title(&ctx), "elephant@1.0.1");
    }

    #[test]
    fn extract_unique_app_names() {
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
        let app_names = ctx.get_latest_pkg_names();
        assert_eq!(app_names.len(), 2);
        assert!(app_names.contains(&"tiger".to_string()));
        assert!(app_names.contains(&"elephant".to_string()));
    }

    #[test]
    fn should_differentiate_v_prefix() {
        let r = vec![
            Release {
                tag_name: "tiger@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.0".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "tiger@1.1.0".to_string(),
                ..Default::default()
            },
        ];
        let ctx = AppContext::new(r);

        let latest_releases = ctx.get_latest_versions();
        let tiger = latest_releases.get("tiger").unwrap();
        let elephant = latest_releases.get("elephant").unwrap();
        println!("{:?}", latest_releases);

        assert_eq!(tiger.has_v_prefix, false);
        // TODO this should return true, debug!
        assert_eq!(elephant.has_v_prefix, true);
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

        let all_versions = extract_all_gh_pkgs_and_versions(releases);
        assert_eq!(all_versions.len(), 2);
        assert_eq!(all_versions.get("tiger").unwrap().len(), 2);
        assert_eq!(all_versions.get("elephant").unwrap().len(), 1);
    }

    #[test]
    fn test_extract_pkgs_and_latest_versions() {
        let r = vec![
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

        let ctx = AppContext::new(r);
        let latest_versions = ctx.get_latest_versions();
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
        let r = vec![
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

        let ctx = AppContext::new(r);
        let latest_versions = ctx.get_latest_versions();
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
