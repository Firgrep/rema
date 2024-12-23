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
        let releases = ctx.get_releases();
        let latest_versions = transformer::extract_pkgs_and_latest_versions(releases.clone());
        let pkgs = ctx.get_pkgs();

        let selected_pkg = cli::select_pkg_name(pkgs)
            .unwrap_or_else(|e| panic!("Failed to select package {:?}", Some(e)));
        ctx.set_selected_package(selected_pkg.clone());

        let selected_pkg_version = latest_versions.get(&selected_pkg).unwrap_or_else(|| {
            panic!("Failed to get version for package: {}", selected_pkg);
        });

        println!(
            "  {} is currently released as version {}",
            selected_pkg.clone().green().underlined(),
            selected_pkg_version.clone().to_string().yellow()
        );

        let selected_bump = cli::select_version_bump(&ctx, selected_pkg_version.clone())
            .unwrap_or_else(|e| panic!("Failed to select version bump {:?}", Some(e)));
        ctx.set_selected_bump(selected_bump.clone());

        let target_version = transformer::bump_version(&ctx, selected_pkg_version, selected_bump);

        println!(
            "Bumped version for {} from {} to {}",
            selected_pkg, selected_pkg_version, target_version
        );
    }

    fn requirements_check() {
        let gh_cli = gh::check_gh_cli().unwrap_or_else(|e| {
            panic!("GitHub CLI check failed: {:?}", Some(e).unwrap());
        });

        if !gh_cli {
            panic!("GitHub CLI is not installed");
        }

        let git = git::check_git().unwrap_or_else(|e| {
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
