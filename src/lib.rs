use api::{gh, git};
use colorize::AnsiColor;

mod api;
mod cli;
mod ctx;
mod transformer;
mod writer;

pub struct Rema {}

impl Rema {
    pub fn run() {
        Self::requirements_check();
        let mut ctx = ctx::create_ctx_with_data();
        let pkgs = ctx.get_pkgs();

        let selected_pkg = cli::select_pkg_name(pkgs)
            .unwrap_or_else(|e| panic!("Failed to select package {:?}", Some(e)));

        ctx.set_selected_package(selected_pkg.clone());

        let latest_versions = ctx.get_latest_versions().clone();

        let selected_pkg_release_info = latest_versions.get(&selected_pkg).unwrap_or_else(|| {
            panic!("Failed to get version for package: {}", selected_pkg);
        });

        println!(
            "  {} is currently released as version {}",
            selected_pkg.clone().green().underlined(),
            selected_pkg_release_info
                .version
                .clone()
                .to_string()
                .yellow()
        );

        let selected_bump = cli::select_version_bump(&ctx)
            .unwrap_or_else(|e| panic!("Failed to select version bump {:?}", Some(e)));

        ctx.set_selected_bump(selected_bump.clone());

        let target_version = transformer::bump_version(&ctx, selected_bump);
        ctx.set_target_version(target_version.clone());

        println!(
            "Bumped version for {} from {} to {}",
            selected_pkg, selected_pkg_release_info.version, target_version
        );

        // TODO retrieve the v_prefix where relevant
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

        git::verify_no_outstanding_commits().unwrap_or_else(|e| {
            panic!("{:?}", Some(e).unwrap());
        });
    }
}
