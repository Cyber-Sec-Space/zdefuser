use std::path::Path;
use walkdir::WalkDir;

pub fn validate_extracted_dir(dir: &Path, max_total_bytes: u64) -> Result<(), String> {
    let mut total_size = 0;

    for entry in WalkDir::new(dir).min_depth(1) {
        let entry = entry.map_err(|e| format!("Walkdir error: {}", e))?;
        let path = entry.path();

        if entry.file_type().is_symlink() {
            return Err(format!(
                "Security violation: Symlink detected at {}",
                path.display()
            ));
        }

        // Host side canonicalization check
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if !canonical.starts_with(dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf())) {
            return Err(format!(
                "Security violation: Extracted file escaped sandbox {}",
                path.display()
            ));
        }

        if entry.file_type().is_file() {
            let metadata = entry
                .metadata()
                .map_err(|e| format!("Metadata error: {}", e))?;
            total_size += metadata.len();
        }

        if total_size > max_total_bytes {
            return Err("Security violation: Total size exceeded during host audit".to_string());
        }
    }

    Ok(())
}
