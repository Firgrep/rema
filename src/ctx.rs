use std::collections::HashMap;

use semver::Version;

use crate::{
    gh::{self, Release},
    read::LocalPackageFiles,
    transform::{self, PreReleaseType, ReleaseInfo, VersionBump},
};

pub struct AppContext {
    local_pkg_files: Option<Vec<LocalPackageFiles>>,
    all_gh_versions: HashMap<String, Vec<ReleaseInfo>>,
    latest_gh_versions: HashMap<String, ReleaseInfo>,
    selected_pkg: Option<String>,
    selected_bump: Option<VersionBump>,
    target_version: Option<ReleaseInfo>,
    gh_generate_release_notes: bool,
}

pub fn create_ctx_with_data() -> AppContext {
    let releases = gh::list_releases().unwrap_or_else(|e| {
        panic!("Failed to list releases {:?}", Some(e));
    });

    AppContext::new(releases)
}

impl AppContext {
    pub fn new(releases: Vec<Release>) -> Self {
        let all_versions = transform::extract_all_gh_pkgs_and_versions(releases.clone());
        let latest_versions = transform::extract_latest_gh_pkgs_and_versions(&all_versions);

        Self {
            all_gh_versions: all_versions,
            latest_gh_versions: latest_versions,
            local_pkg_files: None,
            selected_pkg: None,
            selected_bump: None,
            target_version: None,
            gh_generate_release_notes: true,
        }
    }

    pub fn get_gh_generate_release_notes(&self) -> bool {
        self.gh_generate_release_notes
    }

    pub fn get_pre_for_version_for_selected_pkg(
        &self,
        pre: &str,
        version: &Version,
    ) -> Option<&ReleaseInfo> {
        let pkg_name = self.get_selected_package();

        if pkg_name.is_none() {
            panic!("No package selected");
        }

        self.all_gh_versions
            .get(pkg_name.unwrap())
            .unwrap()
            .iter()
            .find(|rel| {
                rel.version.major == version.major
                    && rel.version.minor == version.minor
                    && rel.version.patch == version.patch
                    && rel.version.pre.as_str().contains(pre)
            })
    }

    pub fn get_selected_package(&self) -> Option<&String> {
        self.selected_pkg.as_ref()
    }

    pub fn get_target_release_info(&self) -> Option<&ReleaseInfo> {
        self.target_version.as_ref()
    }

    pub fn get_latest_pkg_names(&self) -> Vec<String> {
        self.latest_gh_versions
            .iter()
            .map(|(name, info)| {
                if info.local_only {
                    format!("{} (unreleased)", name)
                } else {
                    name.clone()
                }
            })
            .collect()
    }

    pub fn get_latest_versions(&self) -> &HashMap<String, ReleaseInfo> {
        &self.latest_gh_versions
    }

    pub fn set_selected_package(&mut self, pkg_name: String) {
        self.selected_pkg = Some(pkg_name);
    }

    pub fn set_and_match_local_pkg_files(&mut self, local_pkg_files: Vec<LocalPackageFiles>) {
        self.local_pkg_files = Some(local_pkg_files.clone());

        match transform::match_local_pkgs_with_gh_pkgs(
            &mut self.latest_gh_versions,
            &local_pkg_files,
        ) {
            Ok(matched_versions) => {
                self.latest_gh_versions = matched_versions;
            }
            Err(e) => {
                panic!(
                    "Failed to match local packages with GitHub packages: {:?}",
                    e
                );
            }
        }
    }

    pub fn set_selected_bump(&mut self, bump: VersionBump) {
        self.selected_bump = Some(bump);
    }

    pub fn set_target_release_info(&mut self, release_info: ReleaseInfo) {
        self.target_version = Some(release_info);
    }

    pub fn find_existing_prerelease(
        &self,
        pkg_name: &str,
        base_version: &Version,
        pre_type: PreReleaseType,
    ) -> Option<&ReleaseInfo> {
        let pre_str = match pre_type {
            PreReleaseType::Alpha => "alpha",
            PreReleaseType::Beta => "beta",
            PreReleaseType::Rc => "rc",
        };

        self.all_gh_versions
            .get(pkg_name)
            .unwrap()
            .iter()
            .find(|rel| {
                rel.version.major == base_version.major
                    && rel.version.minor == base_version.minor
                    && rel.version.patch == base_version.patch
                    && rel.version.pre.as_str().starts_with(pre_str)
            })
    }
}
