use std::{
    error::Error,
    fs::{self},
};

use serde_json::Value;

use crate::ctx::AppContext;

pub enum WriteTargetResult {
    NoWrites,
    WritesCompleted {
        original_pkg_json: Option<String>,
        original_pkg_json_lock: Option<String>,
    },
}

// TODO
pub fn write_target_release_to_file(ctx: &AppContext) -> Result<WriteTargetResult, Box<dyn Error>> {
    let release_info = ctx.get_target_release_info().unwrap_or_else(|| {
        panic!("No target release info found. Run `bump` command first");
    });
    let local_pkg_files = match &release_info.local_pkg_files {
        Some(files) => files,
        None => return Ok(WriteTargetResult::NoWrites),
    };

    let mut original_pkg_json_contents = String::new();
    let mut original_pkg_json_lock_contents = String::new();

    if let Some(path) = local_pkg_files
        .package_json
        .as_ref()
        .and_then(|pkg| pkg.path.as_ref())
    {
        let path_str = path.as_str();
        let original_contents = fs::read_to_string(path_str)?;

        let mut json: Value = serde_json::from_str(&original_contents)?;

        json["version"] = Value::String(release_info.version.clone().to_string());

        let updated_contents = serde_json::to_string_pretty(&json)?;

        if let Err(write_error) = fs::write(path_str, &updated_contents) {
            // Restore the original contents if writing fails
            fs::write(path_str, original_contents)?;
            println!("Error occurred during writing. Attempted to restore original file...");
            return Err(Box::new(write_error));
        }

        original_pkg_json_contents = original_contents;
    }

    if let Some(path) = local_pkg_files
        .package_lock_json
        .as_ref()
        .and_then(|pkg| pkg.path.as_ref())
    {
        let path_str = path.as_str();
        let original_contents = fs::read_to_string(path_str)?;

        let mut json: Value = serde_json::from_str(&original_contents)?;

        json["version"] = Value::String(release_info.version.clone().to_string());

        let updated_contents = serde_json::to_string_pretty(&json)?;

        if let Err(write_error) = fs::write(path_str, &updated_contents) {
            // Restore the original contents if writing fails
            fs::write(path_str, original_contents)?;
            println!("Error occurred during writing. Attempted to restore original file...");
            return Err(Box::new(write_error));
        }

        original_pkg_json_lock_contents = original_contents;
    }

    Ok(WriteTargetResult::WritesCompleted {
        original_pkg_json: if original_pkg_json_contents.is_empty() {
            None
        } else {
            Some(original_pkg_json_contents)
        },
        original_pkg_json_lock: if original_pkg_json_lock_contents.is_empty() {
            None
        } else {
            Some(original_pkg_json_lock_contents)
        },
    })
}
