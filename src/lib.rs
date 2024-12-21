use colorize::AnsiColor;

pub mod cli;
mod ctx;
mod errors;
pub mod gh;
pub mod transformer;
pub mod writer;

pub struct Rema {}

impl Rema {
    pub fn run() {
        let latest_versions = Rema::get_latest_pkgs_and_versions().unwrap();
        let pkgs = latest_versions.keys().cloned().collect();

        let selected_pkg = cli::select_pkg_name(pkgs)
            .unwrap_or_else(|e| errors::fatal_error("Failed to select package", Some(e)));

        let selected_pkg_version = latest_versions.get(&selected_pkg).unwrap_or_else(|| {
            let msg = format!("Failed to get version for package: {}", selected_pkg);
            errors::fatal_error(&msg, None);
        });

        println!(
            "  {} is currently released as version {}",
            selected_pkg.clone().green().underlined(),
            selected_pkg_version.clone().to_string().yellow()
        );

        let selected_bump = cli::select_version_bump(selected_pkg_version.clone())
            .unwrap_or_else(|e| errors::fatal_error("Failed to select version bump", Some(e)));

        let target_version = transformer::bump_version(selected_pkg_version, selected_bump);

        println!(
            "Bumped version for {} from {} to {}",
            selected_pkg, selected_pkg_version, target_version
        );

        // println!("Latest versions:");
        // for (app_name, version) in &latest_versions {
        //     println!("{}: {}", app_name, version);
        // }
    }

    fn get_latest_pkgs_and_versions(
    ) -> Result<std::collections::HashMap<String, semver::Version>, Box<dyn std::error::Error>>
    {
        let releases = gh::list_releases()?;
        Ok(transformer::extract_pkgs_and_latest_versions(releases))
    }
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
