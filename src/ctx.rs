use std::collections::HashMap;

use semver::Version;

use crate::{
    gh::{self, Release},
    transformer::{extract_pkgs_and_all_versions, PreReleaseType, VersionBump},
};

pub struct AppContext {
    releases: Vec<Release>,
    all_versions: HashMap<String, Vec<Version>>,
    selected_pkg: Option<String>,
    selected_bump: Option<VersionBump>,
}

pub fn create_ctx_with_data() -> AppContext {
    let releases = gh::list_releases().unwrap_or_else(|e| {
        panic!("Failed to list releases {:?}", Some(e));
    });

    AppContext::new(releases)
}

impl AppContext {
    pub fn new(releases: Vec<Release>) -> Self {
        let all_versions = extract_pkgs_and_all_versions(releases.clone());
        Self {
            releases,
            all_versions,
            selected_pkg: None,
            selected_bump: None,
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

    pub fn get_pre_for_version_for_selected_pkg(
        &self,
        pre: &str,
        version: &Version,
    ) -> Option<&Version> {
        let pkg_name = self.get_selected_package();

        if pkg_name.is_none() {
            panic!("No package selected");
        }

        self.all_versions
            .get(pkg_name.unwrap())
            .unwrap()
            .iter()
            .find(|v| {
                v.major == version.major
                    && v.minor == version.minor
                    && v.patch == version.patch
                    && v.pre.as_str().contains(pre)
            })
    }

    pub fn get_selected_package(&self) -> Option<&String> {
        self.selected_pkg.as_ref()
    }

    pub fn get_pkgs(&self) -> Vec<String> {
        self.all_versions.keys().cloned().collect()
    }

    pub fn get_releases(&self) -> &Vec<Release> {
        &self.releases
    }

    pub fn find_existing_prerelease(
        &self,
        pkg_name: &str,
        base_version: &Version,
        pre_type: PreReleaseType,
    ) -> Option<&Version> {
        let pre_str = match pre_type {
            PreReleaseType::Alpha => "alpha",
            PreReleaseType::Beta => "beta",
            PreReleaseType::Rc => "rc",
        };

        self.all_versions.get(pkg_name).unwrap().iter().find(|v| {
            v.major == base_version.major
                && v.minor == base_version.minor
                && v.patch == base_version.patch
                && v.pre.as_str().starts_with(pre_str)
        })
    }
}
