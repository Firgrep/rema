use api::{gh, git};
use colorize::AnsiColor;

mod api;
mod cli;
mod ctx;
mod read;
mod transform;
mod write;

// TODO
// TODO - write and release phase
// ? DONE keep backups of the original files to revert if the process below fails
// ? DONE write new version to local package.json if they exist
// commit and push changes, create the new release
// git pull to sync with remote (get tags)
// error handle and restore original files if above fails
// TODO - command line arguments
// TODO - allow selection of which pre to bump if multiple

/// Rema is a tool to help you manage your releases
pub struct Rema {}

/// Rema is a tool to help you manage your releases
impl Rema {
    /// Run the application
    pub fn run() {
        Self::requirements_check();
        let mut ctx = ctx::create_ctx_with_data();
        if let Some(pkg_files) = read::find_local_pkg_files() {
            ctx.set_and_match_local_pkg_files(pkg_files);
        }
        let pkgs = ctx.get_latest_pkg_names();

        // let local_pkgs = ctx.get_local_pkg_files().unwrap_or_else(|| {
        //     panic!("Failed to get local package files");
        // });
        // println!("Local packages: {:#?}", local_pkgs);

        let selected_pkg = cli::select_pkg_name(pkgs)
            .unwrap_or_else(|e| panic!("Failed to select package {:?}", Some(e)));

        let selected_pkg = selected_pkg.replace("(unreleased)", "").trim().to_string();
        ctx.set_selected_package(selected_pkg.clone());

        let latest_versions = ctx.get_latest_versions().clone();

        let selected_pkg_release_info = latest_versions.get(&selected_pkg).unwrap_or_else(|| {
            panic!("Failed to get version for package: {}", selected_pkg);
        });

        let is_local_only = selected_pkg_release_info.local_only.clone();
        let release_status_mgs = if is_local_only {
            "locally set"
        } else {
            "released"
        };

        println!(
            "  {} is currently {} as version {}",
            selected_pkg.clone().green().underlined(),
            release_status_mgs,
            selected_pkg_release_info.version.clone().to_string().cyan()
        );

        let selected_bump = cli::select_version_bump(&ctx)
            .unwrap_or_else(|e| panic!("Failed to select version bump {:?}", Some(e)));

        ctx.set_selected_bump(selected_bump.clone());

        let target_release_info = transform::bump_version(&ctx, selected_bump);
        ctx.set_target_release_info(target_release_info.clone());

        let initial_release_title = transform::create_release_title(&ctx);

        let target_title =
            cli::input_release_title(initial_release_title.as_str()).unwrap_or_else(|e| {
                panic!("Failed to input release title {:?}", Some(e));
            });

        let target_description = cli::input_release_description(&ctx).unwrap_or_else(|e| {
            panic!("Failed to input release description {:?}", Some(e));
        });

        let is_confirmed = cli::input_confirmation(&ctx).unwrap_or_else(|e| {
            panic!("Error occurred during confirmation {:?}", Some(e));
        });

        if !is_confirmed {
            println!("Aborted");
            return;
        }

        println!(
            "Bumped version for {} from {} to {}",
            selected_pkg, selected_pkg_release_info.version, target_release_info.version
        );

        println!("has v prefix: {}", target_release_info.has_v_prefix);
        println!("title: {}", target_title);
        println!("target_description: {}", target_description);
        println!("local_pkgs: {:#?}", target_release_info.local_pkg_files);
    }

    fn requirements_check() {
        let gh_cli = gh::verify_gh_cli_version().unwrap_or_else(|e| {
            panic!("GitHub CLI check failed: {:?}", Some(e).unwrap());
        });

        if !gh_cli {
            panic!("GitHub CLI is not installed");
        }

        let git = git::verify_git_version().unwrap_or_else(|e| {
            panic!("Git check failed: {:?}", Some(e).unwrap());
        });

        if !git {
            panic!("Git is not installed");
        }

        // TODO
        // git::verify_no_outstanding_commits().unwrap_or_else(|e| {
        //     panic!("{:?}", Some(e).unwrap());
        // });
    }
}
