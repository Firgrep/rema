use std::{fs::File, io::Write, path::Path};

use crate::gh::Release;

// TODO
pub fn _write_releases_to_file(releases: &[Release], file_path: &str) -> Result<(), String> {
    // Serialize releases to JSON
    let json = serde_json::to_string_pretty(releases)
        .map_err(|e| format!("Failed to serialize releases: {}", e))?;

    // Create or overwrite the file
    let path = Path::new(file_path);
    let mut file =
        File::create(path).map_err(|e| format!("Failed to create file {}: {}", file_path, e))?;

    // Write JSON to the file
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write to file {}: {}", file_path, e))?;

    Ok(())
}
