use inquire::Select;
use semver::Version;

use crate::transformer::{BaseVersion, PreReleaseType, VersionBump};

pub fn select_pkg_name(options: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
    let ordered_options = order_pkg_names(options);
    let ans = Select::new("Select which package to manage", ordered_options).prompt()?;
    Ok(ans)
}

fn order_pkg_names(pkgs: Vec<String>) -> Vec<String> {
    let mut pkgs = pkgs;
    pkgs.sort();
    pkgs
}

pub fn select_version_bump(version: Version) -> Result<VersionBump, Box<dyn std::error::Error>> {
    let options = if version.pre.is_empty() {
        vec!["major", "minor", "patch", "create pre-release"]
    } else {
        vec![
            "major",
            "minor",
            "patch",
            "pre-release",
            "create pre-release",
        ]
    };

    let ans = Select::new("Select which version bump to apply", options).prompt()?;
    let ans = match ans {
        "major" => VersionBump::Major,
        "minor" => VersionBump::Minor,
        "patch" => VersionBump::Patch,
        "pre-release" => VersionBump::Pre,
        "create pre-release" => create_pre_release()?,
        _ => panic!("Invalid version bump"),
    };

    Ok(ans)
}

fn create_pre_release() -> Result<VersionBump, Box<dyn std::error::Error>> {
    let pre_base_version = select_pre_release_base_version()?;
    let pre_type = select_pre_release_type()?;
    Ok(VersionBump::PreNew(pre_type, pre_base_version))
}

fn select_pre_release_type() -> Result<PreReleaseType, Box<dyn std::error::Error>> {
    let options = vec!["alpha", "beta", "rc"];
    let ans = Select::new("Select which pre-release type to create", options).prompt()?;
    let ans = match ans {
        "alpha" => PreReleaseType::Alpha,
        "beta" => PreReleaseType::Beta,
        "rc" => PreReleaseType::Rc,
        _ => panic!("Invalid pre-release type"),
    };
    Ok(ans)
}

fn select_pre_release_base_version() -> Result<BaseVersion, Box<dyn std::error::Error>> {
    let options_pre_versions = vec!["major", "minor", "patch"];
    let ans = Select::new(
        "Which version type is this a pre-release for?",
        options_pre_versions,
    )
    .prompt()?;
    let ans = match ans {
        "major" => BaseVersion::Major,
        "minor" => BaseVersion::Minor,
        "patch" => BaseVersion::Patch,
        _ => panic!("Invalid base version"),
    };
    Ok(ans)
}
