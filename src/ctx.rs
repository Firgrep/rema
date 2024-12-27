use std::collections::HashMap;

use semver::Version;

use crate::{
    gh::{self, Release},
    transform::{self, PreReleaseType, ReleaseInfo, VersionBump},
};

pub struct AppContext {
    all_versions: HashMap<String, Vec<ReleaseInfo>>,
    latest_versions: HashMap<String, ReleaseInfo>,
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
        let all_versions = transform::extract_pkgs_and_all_versions(releases.clone());
        let latest_versions = transform::extract_pkgs_and_latest_versions(&all_versions);

        Self {
            all_versions,
            latest_versions,
            selected_pkg: None,
            selected_bump: None,
            target_version: None,
            gh_generate_release_notes: true,
        }
    }

    pub fn set_selected_package(&mut self, pkg_name: String) {
        if self.all_versions.contains_key(&pkg_name) {
            self.selected_pkg = Some(pkg_name);
        } else {
            panic!("Package not found: {}", pkg_name);
        }
    }

    pub fn set_selected_bump(&mut self, bump: VersionBump) {
        self.selected_bump = Some(bump);
    }

    pub fn set_target_release_info(&mut self, release_info: ReleaseInfo) {
        self.target_version = Some(release_info);
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

        self.all_versions
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

    pub fn get_pkgs(&self) -> Vec<String> {
        self.all_versions.keys().cloned().collect()
    }

    pub fn get_latest_versions(&self) -> &HashMap<String, ReleaseInfo> {
        &self.latest_versions
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

        self.all_versions.get(pkg_name).unwrap().iter().find(|rel| {
            rel.version.major == base_version.major
                && rel.version.minor == base_version.minor
                && rel.version.patch == base_version.patch
                && rel.version.pre.as_str().starts_with(pre_str)
        })
    }
}
