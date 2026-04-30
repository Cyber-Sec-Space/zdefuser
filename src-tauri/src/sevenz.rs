use std::fs::{self, File};
use std::path::{Component, Path};
use tokio::sync::mpsc;

struct SecurityContext {
    max_ratio: u64,
    max_total_bytes: u64,
    max_files: usize,
    extracted_bytes: u64,
    extracted_files: usize,
    blocked_files: usize,
    archive_size: u64,
}

impl SecurityContext {
    fn new(max_ratio: u64, max_total_bytes: u64, max_files: usize, archive_size: u64) -> Self {
        Self {
            max_ratio,
            max_total_bytes,
            max_files,
            archive_size,
            extracted_bytes: 0,
            extracted_files: 0,
            blocked_files: 0,
        }
    }

    fn record_and_check(&mut self, _compressed_size: u64, uncompressed_size: u64) -> Result<(), &'static str> {
        self.extracted_files += 1;
        self.extracted_bytes += uncompressed_size;

        if self.extracted_files > self.max_files {
            return Err("TOO_MANY_FILES");
        }
        if self.extracted_bytes > self.max_total_bytes {
            return Err("TOO_LARGE");
        }
        
        if let Some(global_ratio) = self.extracted_bytes.checked_div(self.archive_size) {
            if global_ratio > self.max_ratio {
                return Err("HIGH_RATIO_ZIP_BOMB");
            }
        }
        
        Ok(())
    }

    pub fn sanitize_path(filepath: &str) -> String {
        let path = Path::new(filepath);
        let mut sanitized = std::path::PathBuf::new();

        for component in path.components() {
            if let Component::Normal(c) = component {
                sanitized.push(c);
            }
        }

        let mut result = sanitized.to_string_lossy().into_owned();

        let bad_chars = [
            '\0', '\x08', '\x09', '\x0A', '\x0D', 
            '\u{202E}', '\u{200F}', '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202F}'
        ];
        result.retain(|c| !bad_chars.contains(&c));

        if result.is_empty() {
            "unnamed_sanitized_file".to_string()
        } else {
            result
        }
    }
}

pub fn extract_7z(
    archive_path: &str,
    output_dir: &Path,
    password: Option<&str>,
    max_ratio: u64,
    max_total_bytes: u64,
    max_files: usize,
    tx: mpsc::Sender<String>,
) -> Result<(), String> {
    println!("extract_7z called with {}", archive_path);
    let file = File::open(archive_path).map_err(|e| format!("Failed to open 7z: {}", e))?;
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut sec_ctx = SecurityContext::new(max_ratio, max_total_bytes, max_files, len);

    let pwd = match password {
        Some(p) => sevenz_rust::Password::from(p),
        None => sevenz_rust::Password::empty(),
    };

    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut archive = match sevenz_rust::SevenZReader::new(file, len, pwd) {
        Ok(a) => a,
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.to_lowercase().contains("password") {
                let _ = tx.blocking_send(format!(
                    r#"{{"type": "error", "code": "PASSWORD_REQUIRED", "details": "Password required for encrypted archive: {}"}}"#,
                    err_msg.replace("\"", "\\\"")
                ));
            } else {
                let _ = tx.blocking_send(format!(
                    r#"{{"type": "error", "code": "EXTRACT_FAILED", "details": "Invalid 7z archive: {}"}}"#,
                    err_msg.replace("\"", "\\\"")
                ));
            }
            return Err(format!("Invalid 7z or Password required: {}", e));
        }
    };

    let mut count = 0;
    
    // We must handle the Result properly without capturing tx asynchronously in for_each_entries
    let mut entries_result = Ok(());

    println!("Starting for_each_entries");
    archive
        .for_each_entries(|entry, reader| {
            let filename_str = entry.name();
            if filename_str.is_empty() {
                return Ok(true);
            }

            let sanitized_filename = SecurityContext::sanitize_path(filename_str);
            if sanitized_filename != filename_str {
                let _ = tx.blocking_send(format!(
                    r#"{{"type": "warning", "code": "PATH_SANITIZED", "file": "{}", "details": "Path traversal or dangerous characters neutralized"}}"#,
                    filename_str.replace("\"", "\\\"")
                ));
                sec_ctx.blocked_files += 1;
            }

            let uncompressed_size = entry.size();

            if entry.has_stream() {
                if let Err(err_code) = sec_ctx.record_and_check(0, uncompressed_size) {
                    let _ = tx.blocking_send(format!(
                        r#"{{"type": "error", "code": "{}", "details": "limit exceeded"}}"#,
                        err_code
                    ));
                    entries_result = Err("Aborted due to security threshold".to_string());
                    return Ok(false); // Stop extraction
                }
            }

            println!("Extracting entry: {}", sanitized_filename);
            count += 1;
            let out_path = output_dir.join(&sanitized_filename);

            if entry.is_directory() {
                fs::create_dir_all(&out_path).ok();
            } else if entry.has_stream() {
                if let Some(p) = out_path.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).ok();
                    }
                }
                
                if let Ok(mut outfile) = File::create(&out_path) {
                    use std::io::{Read, Write};
                    let mut reader_limited = reader.take(uncompressed_size);
                    let mut buf = [0u8; 65536]; // 64KB buffer
                    let mut file_written = 0;
                    let mut last_reported = 0;

                    loop {
                        match reader_limited.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                if outfile.write_all(&buf[..n]).is_err() {
                                    let _ = tx.blocking_send(
                                        r#"{"type": "error", "code": "EXTRACT_FAILED", "details": "Failed to write to disk"}"#.to_string()
                                    );
                                    entries_result = Err("Failed to write to disk".to_string());
                                    break;
                                }
                                file_written += n as u64;

                                // Report progress every 10MB to keep UI responsive
                                if file_written - last_reported > 10 * 1024 * 1024 {
                                    last_reported = file_written;
                                    let _ = tx.blocking_send(format!(
                                        r#"{{"type": "progress", "current": {}, "total": 0, "file": "{}", "bytes": {}}}"#,
                                        count,
                                        sanitized_filename.replace("\"", "\\\""),
                                        file_written
                                    ));
                                }
                            }
                            Err(e) => {
                                let err_msg = e.to_string();
                                if err_msg.to_lowercase().contains("password") || err_msg.to_lowercase().contains("checksum") {
                                    let _ = tx.blocking_send(
                                        r#"{"type": "error", "code": "PASSWORD_REQUIRED", "details": "Password required or invalid for encrypted archive."}"#.to_string()
                                    );
                                } else {
                                    let _ = tx.blocking_send(format!(
                                        r#"{{"type": "error", "code": "EXTRACT_FAILED", "details": "{}"}}"#,
                                        err_msg.replace("\"", "\\\"")
                                    ));
                                }
                                entries_result = Err(format!("Read error: {}", e));
                                break;
                            }
                        }
                    }

                    if entries_result.is_err() {
                        return Ok(false);
                    }

                    // Final progress report for this file
                    let _ = tx.blocking_send(format!(
                        r#"{{"type": "progress", "current": {}, "total": 0, "file": "{}", "bytes": {}}}"#,
                        count,
                        sanitized_filename.replace("\"", "\\\""),
                        file_written
                    ));
                }
                }

            Ok(true)
        })
        .map_err(|e| format!("Extract 7z failed: {}", e))?;

    entries_result?;

    let _ = tx.blocking_send(format!(
        r#"{{"type": "complete", "files_extracted": {}, "files_blocked": {}, "total_bytes": {}}}"#,
        sec_ctx.extracted_files, sec_ctx.blocked_files, sec_ctx.extracted_bytes
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_7z_password_hang() {
        let file = File::open("../tests/payloads/11_7z_encrypted.7z").unwrap();
        let len = file.metadata().unwrap().len();
        let pwd = sevenz_rust::Password::empty();
        
        let handle = tokio::task::spawn_blocking(move || {
            sevenz_rust::SevenZReader::new(file, len, pwd).is_err()
        });

        assert!(handle.await.unwrap());
    }
}
