use inquire::Select;
use semver::Version;

use crate::{
    ctx::AppContext,
    transformer::{PreReleaseType, PreReleaseVersionBump, VersionBump},
};

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

pub fn select_version_bump(
    ctx: &AppContext,
    existing_version: Version,
) -> Result<VersionBump, Box<dyn std::error::Error>> {
    let options = if existing_version.pre.is_empty() {
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
        PRE_NEW => create_pre_release(ctx, existing_version)?,
        _ => panic!("Invalid version bump"),
    };

    Ok(ans)
}

fn create_pre_release(
    ctx: &AppContext,
    existing_version: Version,
) -> Result<VersionBump, Box<dyn std::error::Error>> {
    let pre_base_version = select_pre_release_base_version()?;
    let pre_type = select_pre_release_type(ctx, &pre_base_version, existing_version)?;
    Ok(VersionBump::PreNew(pre_type, pre_base_version))
}

fn select_pre_release_type(
    ctx: &AppContext,
    pre_base_version: &PreReleaseVersionBump,
    existing_version: Version,
) -> Result<PreReleaseType, Box<dyn std::error::Error>> {
    let mut options = vec![];

    match pre_base_version {
        // If user wants to retain the pre-release version, we need to check if there
        // already exists pre-release types for this version, so that no new pre-releases
        // are created that are of lower version than the existing ones.
        PreReleaseVersionBump::Retain => {
            let alpha = ctx.get_pre_for_version_for_selected_pkg(ALPHA, &existing_version);
            if !alpha.is_some() {
                options.push(ALPHA);
            }
            let beta = ctx.get_pre_for_version_for_selected_pkg(BETA, &existing_version);
            if !beta.is_some() {
                options.push(BETA);
            }
            let rc = ctx.get_pre_for_version_for_selected_pkg(RC, &existing_version);
            if !rc.is_some() {
                options.push(RC);
            }
        }
        _ => {
            options.push(ALPHA);
            options.push(BETA);
            options.push(RC);
        }
    }

    if options.is_empty() {
        return Err("No new pre-release types available for this version. Try bumping existing ones or bump the major, minor or patch".into());
    }

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
        "Which version number to bump for this pre-release?",
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

#[cfg(test)]
mod tests {
    use crate::gh::Release;

    use super::*;

    #[test]
    fn should_error_when_no_options_are_available_for_select_pre_release_type() {
        let test_releases = vec![
            Release {
                tag_name: "elephant@v1.0.1-beta.3".to_string(),
                ..Default::default()
            },
            Release {
                tag_name: "elephant@v1.0.1-alpha.3".to_string(),
                ..Default::default()
            },
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

        let existing_version = Version::new(1, 0, 1);
        let pre_base_version = PreReleaseVersionBump::Retain;

        let result = select_pre_release_type(&ctx, &pre_base_version, existing_version);

        assert!(result.is_err());
    }
}
