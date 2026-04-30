use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn move_and_strip_permissions(source_dir: &Path, target_dir: &Path) -> Result<(), String> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)
            .map_err(|e| format!("Failed to create target dir: {}", e))?;
    }

    for entry in WalkDir::new(source_dir).min_depth(1) {
        let entry = entry.map_err(|e| format!("Walkdir error: {}", e))?;
        let rel_path = entry.path().strip_prefix(source_dir).unwrap();
        let target_path = target_dir.join(rel_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path).ok();
        } else if entry.file_type().is_file() {
            if let Some(p) = target_path.parent() {
                fs::create_dir_all(p).ok();
            }
            // Try rename (move) first for instant zero-copy. Fallback to copy+remove if crossing filesystems.
            if fs::rename(entry.path(), &target_path).is_err() {
                fs::copy(entry.path(), &target_path)
                    .map_err(|e| format!("Copy failed for {}: {}", rel_path.display(), e))?;
                fs::remove_file(entry.path()).ok();
            }

            // Strip executable permissions
            if let Ok(metadata) = fs::metadata(&target_path) {
                let mut perms = metadata.permissions();
                #[cfg(unix)]
                {
                    let mode = perms.mode();
                    // Remove executable bits (0o111) AND all special bits like SUID/SGID (0o7000)
                    // We only allow read and write bits (0o666)
                    let new_mode = mode & 0o666;
                    perms.set_mode(new_mode);
                    let _ = fs::set_permissions(&target_path, perms);
                }
                #[cfg(not(unix))]
                {
                    let _ = perms; // Ignore unused
                }
            }
        }
    }

    Ok(())
}
