use colorize::AnsiColor;
use inquire::{
    ui::{Color, RenderConfig, Styled},
    Confirm, Editor, Select,
};
use semver::Version;

use crate::{
    ctx::AppContext,
    transform::{PreReleaseType, PreReleaseVersionBump, VersionBump},
};

const MAJOR: &str = "major";
const MINOR: &str = "minor";
const PATCH: &str = "patch";
const RETAIN: &str = "retain version";
const USE_LOCAL: &str = "use unreleased local version";
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

pub fn select_version_bump(ctx: &AppContext) -> Result<VersionBump, Box<dyn std::error::Error>> {
    let releases = ctx.get_latest_versions();
    let selected_pkg = ctx.get_selected_package().unwrap();
    let selected_pkg_release_info = releases.get(selected_pkg).unwrap();

    let existing_version = selected_pkg_release_info.version.clone();

    let mut options = if existing_version.pre.is_empty() {
        vec![MAJOR, MINOR, PATCH, PRE_NEW]
    } else {
        vec![MAJOR, MINOR, PATCH, PRE, PRE_NEW]
    };

    if selected_pkg_release_info.local_only {
        options.push(USE_LOCAL)
    }

    let ans = Select::new("Select which version bump to apply", options).prompt()?;
    let ans = match ans {
        MAJOR => VersionBump::Major,
        MINOR => VersionBump::Minor,
        PATCH => VersionBump::Patch,
        PRE => VersionBump::Pre,
        PRE_NEW => create_pre_release(ctx, existing_version)?,
        USE_LOCAL => VersionBump::RetainIfUnreleased,
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

pub fn input_release_title(initial_title: &str) -> Result<String, Box<dyn std::error::Error>> {
    let format_title = format!(
        "Title (currently set to: {}):",
        initial_title.to_string().yellow()
    );
    let title = Editor::new(format_title.as_str())
        .with_predefined_text(initial_title)
        .prompt()?;
    Ok(title)
}

pub fn input_release_description(ctx: &AppContext) -> Result<String, Box<dyn std::error::Error>> {
    let gh_msg = if ctx.get_gh_generate_release_notes() {
        "(gh auto generate will not overwrite title or description)"
    } else {
        ""
    };
    let desc_msg = format!("Description {}", gh_msg.grey());
    let description = Editor::new(desc_msg.as_str())
        .with_formatter(&|submission| {
            let char_count = submission.chars().count();
            if char_count == 0 {
                String::from("<skipped>")
            } else if char_count <= 20 {
                submission.into()
            } else {
                let mut substr: String = submission.chars().take(17).collect();
                substr.push_str("...");
                substr
            }
        })
        .with_render_config(description_render_config())
        .prompt()?;
    Ok(description)
}

pub fn input_confirmation(ctx: &AppContext) -> Result<bool, Box<dyn std::error::Error>> {
    let selected_pkg = ctx.get_selected_package().unwrap().clone();
    let target_release = ctx.get_target_release_info().unwrap().clone();

    let help_msg = get_confirmation_help_msg(ctx);

    let msg = format!(
        "Are you sure you want to release {} version {}?",
        selected_pkg.to_string().cyan(),
        target_release.version.to_string().green()
    );
    let ans = Confirm::new(msg.as_str())
        .with_default(true)
        .with_help_message(help_msg.as_str())
        .prompt()?;

    Ok(ans)
}

fn get_confirmation_help_msg(ctx: &AppContext) -> String {
    let target_release = ctx.get_target_release_info().unwrap().clone();

    let mut help_msg = String::new();

    let local_pkg_files = match target_release.local_pkg_files {
        None => return help_msg,
        Some(files) => files,
    };

    local_pkg_files
        .package_json
        .and_then(|pkg| pkg.name.and_then(|_| pkg.path))
        .map(|path| help_msg.push_str(&format!("Will update {}", path)));

    local_pkg_files
        .package_lock_json
        .and_then(|pkg| pkg.name.and_then(|_| pkg.path))
        .map(|path| help_msg.push_str(&format!("\nWill update {}", path)));

    help_msg
}

fn description_render_config() -> RenderConfig<'static> {
    RenderConfig::default()
        .with_canceled_prompt_indicator(Styled::new("<skipped>").with_fg(Color::DarkYellow))
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
