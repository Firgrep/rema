use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

/// Struct to hold relevant package.json fields
#[derive(Deserialize, Debug, Clone)]
pub struct PackageJson {
    pub name: Option<String>,
    pub version: Option<String>,
    pub path: Option<String>,
}

/// Struct to hold relevant package-lock.json fields
#[derive(Deserialize, Debug, Clone)]
pub struct PackageLockJson {
    pub name: Option<String>,
    pub version: Option<String>,
    pub path: Option<String>,
}

/// Struct to hold package.json and package-lock.json
#[derive(Deserialize, Debug, Clone)]
pub struct LocalPackageFiles {
    pub name: Option<String>,
    pub package_json: Option<PackageJson>,
    pub package_lock_json: Option<PackageLockJson>,
}

pub fn find_local_pkg_files() -> Option<Vec<LocalPackageFiles>> {
    let current_dir = env::current_dir().expect("Failed to get current directory");

    println!("Scanning for package.json files in {:?}", current_dir);

    let mut results = Vec::<LocalPackageFiles>::new();

    scan_for_package_json(&current_dir, &mut results);
    scan_for_package_lock_json(&current_dir, &mut results);

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

/// Recursively scans directories for package.json files, skipping node_modules
fn scan_for_package_json(dir: &Path, results: &mut Vec<LocalPackageFiles>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path
                .file_name()
                .map_or(false, |name| name == "node_modules")
            {
                continue;
            }

            if path.is_dir() {
                // Recurse into subdirectories
                scan_for_package_json(&path, results);
            } else if path
                .file_name()
                .map_or(false, |name| name == "package.json")
            {
                // Parse package.json
                if let Some(package_json) = parse_package_json(&path) {
                    let pkg_files = LocalPackageFiles {
                        name: package_json.name.clone(),
                        package_json: Some(package_json),
                        package_lock_json: None,
                    };
                    results.push(pkg_files);
                }
            }
        }
    }
}

/// Recursively scans directories for package-lock.json files, skipping node_modules
/// Will only add them if a package.json that matches the name is found
fn scan_for_package_lock_json(dir: &Path, results: &mut Vec<LocalPackageFiles>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path
                .file_name()
                .map_or(false, |name| name == "node_modules")
            {
                continue;
            }

            if path.is_dir() {
                // Recurse into subdirectories
                scan_for_package_lock_json(&path, results);
            } else if path
                .file_name()
                .map_or(false, |name| name == "package-lock.json")
            {
                // Parse package-lock.json
                if let Some(package_lock_json) = parse_package_lock_json(&path) {
                    let name = package_lock_json.name.clone();
                    if let Some(pkg_files) = results.iter_mut().find(|x| x.name == name) {
                        if pkg_files
                            .package_json
                            .as_ref()
                            .and_then(|pj| pj.version.clone())
                            == package_lock_json.version
                        {
                            pkg_files.package_lock_json = Some(package_lock_json);
                        } else {
                            panic!(
                                "Found package-lock.json with no matching package.json or version. {:?}",
                                pkg_files.package_json.as_ref().unwrap().path.clone().unwrap_or_default()
                            )
                        }
                    }
                }
            }
        }
    }
}

/// Function to parse package.json
fn parse_package_json(path: &Path) -> Option<PackageJson> {
    let content = fs::read_to_string(path).ok()?;
    let mut package_json: PackageJson = serde_json::from_str(&content).ok()?;
    package_json.path = Some(path.to_string_lossy().to_string());
    Some(package_json)
}

/// Function to parse package-lock.json
fn parse_package_lock_json(path: &Path) -> Option<PackageLockJson> {
    let content = fs::read_to_string(path).ok()?;
    let mut package_lock_json: PackageLockJson = serde_json::from_str(&content).ok()?;
    package_lock_json.path = Some(path.to_string_lossy().to_string());
    Some(package_lock_json)
}
