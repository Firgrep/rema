use inquire::Select;
use semver::Version;

use crate::transformer::{PreReleaseType, PreReleaseVersionBump, VersionBump};

const MAJOR: &str = "major";
const MINOR: &str = "minor";
const PATCH: &str = "patch";
const RETAIN: &str = "retain version";
const PRE: &str = "pre-release";
const PRE_NEW: &str = "create new pre-release";
const ALPHA: &str = "alpha";
const BETA: &str = "beta";
const RC: &str = "rc";

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
        vec![MAJOR, MINOR, PATCH, PRE_NEW]
    } else {
        vec![MAJOR, MINOR, PATCH, PRE, PRE_NEW]
    };

    let ans = Select::new("Select which version bump to apply", options).prompt()?;
    let ans = match ans {
        MAJOR => VersionBump::Major,
        MINOR => VersionBump::Minor,
        PATCH => VersionBump::Patch,
        PRE => VersionBump::Pre,
        PRE_NEW => create_pre_release()?,
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
    let options = vec![ALPHA, BETA, RC];
    let ans = Select::new("Select which pre-release type to create", options).prompt()?;
    let ans = match ans {
        ALPHA => PreReleaseType::Alpha,
        BETA => PreReleaseType::Beta,
        RC => PreReleaseType::Rc,
        _ => panic!("Invalid pre-release type"),
    };
    Ok(ans)
}

fn select_pre_release_base_version() -> Result<PreReleaseVersionBump, Box<dyn std::error::Error>> {
    let options_pre_versions = vec![MAJOR, MINOR, PATCH, RETAIN];
    let ans = Select::new(
        "Which version to bump for this pre-release?",
        options_pre_versions,
    )
    .prompt()?;
    let ans = match ans {
        MAJOR => PreReleaseVersionBump::Major,
        MINOR => PreReleaseVersionBump::Minor,
        PATCH => PreReleaseVersionBump::Patch,
        RETAIN => PreReleaseVersionBump::Retain,
        _ => panic!("Invalid base version"),
    };
    Ok(ans)
}
