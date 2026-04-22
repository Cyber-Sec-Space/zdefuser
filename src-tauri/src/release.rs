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
                fs::create_dir_all(&p).ok();
            }
            fs::copy(entry.path(), &target_path)
                .map_err(|e| format!("Copy failed for {}: {}", rel_path.display(), e))?;

            // Strip executable permissions
            let mut perms = fs::metadata(&target_path).unwrap().permissions();
            #[cfg(unix)]
            {
                let mode = perms.mode();
                // Remove executable bits for everyone (owner, group, others)
                // e.g. 0o755 (rwxr-xr-x) -> 0o644 (rw-r--r--)
                let new_mode = mode & !0o111;
                perms.set_mode(new_mode);
                let _ = fs::set_permissions(&target_path, perms);
            }
            #[cfg(not(unix))]
            {
                // In windows, we can mark read-only if we wanted to restrict execution somewhat,
                // but usually the execute bit is enough on unix.
                let _ = perms; // Ignore unused
            }
        }
    }

    Ok(())
}
