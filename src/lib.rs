use colorize::AnsiColor;

mod cli;
mod ctx;
mod gh;
mod transformer;
mod writer;

pub struct Rema {}

impl Rema {
    pub fn run() {
        let mut ctx = ctx::create_ctx();
        let releases = ctx.get_releases();
        let latest_versions = transformer::extract_pkgs_and_latest_versions(releases.clone());
        let pkgs = latest_versions.keys().cloned().collect();

        let selected_pkg = cli::select_pkg_name(pkgs)
            .unwrap_or_else(|e| panic!("Failed to select package {:?}", Some(e)));
        ctx.select_package(selected_pkg.clone());

        let selected_pkg_version = latest_versions.get(&selected_pkg).unwrap_or_else(|| {
            panic!("Failed to get version for package: {}", selected_pkg);
        });

        println!(
            "  {} is currently released as version {}",
            selected_pkg.clone().green().underlined(),
            selected_pkg_version.clone().to_string().yellow()
        );

        let selected_bump = cli::select_version_bump(selected_pkg_version.clone())
            .unwrap_or_else(|e| panic!("Failed to select version bump {:?}", Some(e)));

        let target_version = transformer::bump_version(&ctx, selected_pkg_version, selected_bump);

        println!(
            "Bumped version for {} from {} to {}",
            selected_pkg, selected_pkg_version, target_version
        );

        // println!("Latest versions:");
        // for (app_name, version) in &latest_versions {
        //     println!("{}: {}", app_name, version);
        // }
    }

    // fn get_latest_pkgs_and_versions(
    // ) -> Result<std::collections::HashMap<String, semver::Version>, Box<dyn std::error::Error>>
    // {
    //     let releases = gh::list_releases()?;
    //     Ok(transformer::extract_pkgs_and_latest_versions(releases))
    // }
}

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
