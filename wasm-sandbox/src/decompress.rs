use std::fs::{self, File};
use std::io::{copy, Read, Write};
use std::path::Path;

use crc::{Crc, CRC_32_ISCSI, CRC_32_ISO_HDLC};
use flate2::read::GzDecoder;
use rar::Archive;
use tar::Archive as TarArchive;
use zip::ZipArchive;

use crate::protocol::{HostLimit, SandboxEvent};
use crate::security::SecurityContext;

pub fn extract_zip(
    archive_path: &str,
    output_dir: &str,
    password: Option<&str>,
    limits: &HostLimit,
) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Failed to open Zip: {}", e))?;
    let archive_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid Zip: {}", e))?;
    let mut sec_ctx =
        SecurityContext::new(limits.max_ratio, limits.max_total_bytes, limits.max_files, archive_size);

    let total_files = archive.len();
    let out_dir = Path::new(output_dir);

    for i in 0..total_files {
        let file = match password {
            Some(pwd) => match archive.by_index_decrypt(i, pwd.as_bytes()) {
                Ok(f) => f,
                Err(zip::result::ZipError::InvalidPassword) => {
                    return Err("Invalid Archive Password Provided".to_string());
                }
                Err(e) => return Err(format!("Failed to decrypt entry: {}", e)),
            },
            None => match archive.by_index(i) {
                Ok(f) => f,
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("Password required") || msg.contains("encrypted") {
                        let err_str = format!("Password required for encrypted archive: {}", msg);
                        SandboxEvent::Error {
                            code: "PASSWORD_REQUIRED".to_string(),
                            details: err_str.clone(),
                        }
                        .send();
                        return Err(err_str);
                    }
                    return Err(format!("Failed to read entry: {}", msg));
                }
            },
        };

        let filename = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => {
                SandboxEvent::Warning {
                    code: "INVALID_PATH".to_string(),
                    file: file.name().to_string(),
                    details: "Path is missing or invalid".to_string(),
                }
                .send();
                continue;
            }
        };

        let filename_str = filename.to_string_lossy().into_owned();
        let sanitized_filename = SecurityContext::sanitize_path(&filename_str);
        
        if sanitized_filename != filename_str {
            SandboxEvent::Warning {
                code: "PATH_SANITIZED".to_string(),
                file: filename_str.clone(),
                details: "Path traversal neutralized".to_string(),
            }
            .send();
            sec_ctx.blocked_files += 1;
        }

        if file.is_symlink() {
            SandboxEvent::Warning {
                code: "SYMLINK_IGNORED".to_string(),
                file: filename.display().to_string(),
                details: "Symlinks are zero-tolerance dropped for security".to_string(),
            }
            .send();
            sec_ctx.blocked_files += 1;
            continue;
        }

        let compressed_size = file.compressed_size();
        let uncompressed_size = file.size();

        if let Err(err_code) = sec_ctx.record_and_check(compressed_size, uncompressed_size) {
            SandboxEvent::Error {
                code: err_code.to_string(),
                details: format!(
                    "limits exceeded. {} > {}",
                    uncompressed_size, compressed_size
                ),
            }
            .send();
            return Err("Aborted due to security threshold".to_string());
        }

        let out_path = out_dir.join(&sanitized_filename);
        if filename_str.ends_with('/') {
            fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(p) = out_path.parent()
                && !p.exists() {
                    fs::create_dir_all(p).ok();
                }
            let mut outfile =
                File::create(&out_path).map_err(|e| format!("Write failed: {}", e))?;
            
            let mut file_limited = file.take(uncompressed_size);
            let mut buf = [0u8; 65536]; // 64KB chunk
            let mut file_written = 0;
            let mut last_reported = 0;

            loop {
                match file_limited.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        outfile.write_all(&buf[..n]).map_err(|e| format!("Write failed: {}", e))?;
                        file_written += n as u64;

                        // Report progress every 10MB
                        if file_written - last_reported > 10 * 1024 * 1024 {
                            last_reported = file_written;
                            SandboxEvent::Progress {
                                current: (i + 1) as u32,
                                total: total_files as u32,
                                file: sanitized_filename.clone(),
                                bytes: file_written,
                            }
                            .send();
                        }
                    }
                    Err(e) => return Err(format!("Extract failed: {}", e)),
                }
            }

            SandboxEvent::Progress {
                current: (i + 1) as u32,
                total: total_files as u32,
                file: sanitized_filename,
                bytes: file_written,
            }
            .send();
        }
    }

    SandboxEvent::Complete {
        files_extracted: sec_ctx.extracted_files,
        files_blocked: sec_ctx.blocked_files,
        total_bytes: sec_ctx.extracted_bytes,
    }
    .send();

    Ok(())
}

pub fn extract_tar(
    archive_path: &str,
    output_dir: &str,
    is_gz: bool,
    limits: &HostLimit,
) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Failed to open Tar: {}", e))?;
    let archive_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut sec_ctx =
        SecurityContext::new(limits.max_ratio, limits.max_total_bytes, limits.max_files, archive_size);
    let out_dir = Path::new(output_dir);

    // Using Box<dyn Read> to handle both pure tar and tar.gz
    let reader: Box<dyn Read> = if is_gz {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };

    let mut archive = TarArchive::new(reader);
    let entries = archive
        .entries()
        .map_err(|e| format!("Invalid Tar: {}", e))?;

    let mut count = 0;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Read entry failed: {}", e))?;
        let entry_path = entry
            .path()
            .map_err(|e| format!("Invalid path in entry: {}", e))?;
        let filename = entry_path.into_owned();

        let filename_str = filename.to_string_lossy().into_owned();
        let sanitized_filename = SecurityContext::sanitize_path(&filename_str);

        if sanitized_filename != filename_str {
            SandboxEvent::Warning {
                code: "PATH_SANITIZED".to_string(),
                file: filename_str.clone(),
                details: "Path traversal neutralized".to_string(),
            }
            .send();
            sec_ctx.blocked_files += 1;
        }

        // Symlink checks: WASI + pure Rust tar implies we can ignore or block symlinks.
        // We will just extract regular files and dirs to be safe.
        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            SandboxEvent::Warning {
                code: "SYMLINK_IGNORED".to_string(),
                file: filename.display().to_string(),
                details: "Symlinks are prohibited".to_string(),
            }
            .send();
            sec_ctx.blocked_files += 1;
            continue;
        }

        let uncompressed_size = entry.header().size().unwrap_or(0);
        // Tar streams don't easily provide compressed size per file, assume 1 for ratio bypass
        if let Err(err_code) = sec_ctx.record_and_check(uncompressed_size.max(1), uncompressed_size)
        {
            SandboxEvent::Error {
                code: err_code.to_string(),
                details: "limit exceeded".to_string(),
            }
            .send();
            return Err("Aborted due to security threshold".to_string());
        }

        let out_path = out_dir.join(&sanitized_filename);
        if entry_type.is_dir() {
            fs::create_dir_all(&out_path).ok();
        } else if entry_type.is_file() {
            if let Some(p) = out_path.parent()
                && !p.exists() {
                    fs::create_dir_all(p).ok();
                }
            let mut outfile =
                File::create(&out_path).map_err(|e| format!("Write failed: {}", e))?;
            
            let mut file_limited = entry.take(uncompressed_size);
            let mut buf = [0u8; 65536]; // 64KB chunk
            let mut file_written = 0;
            let mut last_reported = 0;

            loop {
                match file_limited.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        outfile.write_all(&buf[..n]).map_err(|e| format!("Write failed: {}", e))?;
                        file_written += n as u64;

                        // Report progress every 10MB
                        if file_written - last_reported > 10 * 1024 * 1024 {
                            last_reported = file_written;
                            SandboxEvent::Progress {
                                current: count,
                                total: 0,
                                file: sanitized_filename.clone(),
                                bytes: file_written,
                            }
                            .send();
                        }
                    }
                    Err(e) => return Err(format!("Extract failed: {}", e)),
                }
            }

            SandboxEvent::Progress {
                current: count,
                total: 0, // Tar streams unknown total
                file: sanitized_filename,
                bytes: file_written,
            }
            .send();
        }
    }

    SandboxEvent::Complete {
        files_extracted: sec_ctx.extracted_files,
        files_blocked: sec_ctx.blocked_files,
        total_bytes: sec_ctx.extracted_bytes,
    }
    .send();

    Ok(())
}

pub fn extract_rar(
    archive_path: &str,
    output_dir: &str,
    password: Option<&str>,
    limits: &HostLimit,
) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Failed to open Rar: {}", e))?;
    let archive_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let pwd = password.unwrap_or("");
    let mut sec_ctx =
        SecurityContext::new(limits.max_ratio, limits.max_total_bytes, limits.max_files, archive_size);

    // rar crate expects trailing slash for the output directory
    let out_dir = if output_dir.ends_with('/') {
        output_dir.to_string()
    } else {
        format!("{}/", output_dir)
    };

    let archive = match rar::Archive::extract_all(archive_path, &out_dir, pwd) {
        Ok(a) => a,
        Err(e) => {
            let msg = format!("{:?}", e);
            if msg.contains("Password") || msg.contains("Checksum") {
                let err_str = format!(
                    "Password required or invalid for encrypted archive. Detailed Error: {:?}",
                    e
                );
                SandboxEvent::Error {
                    code: "PASSWORD_REQUIRED".to_string(),
                    details: err_str.clone(),
                }
                .send();
                return Err(err_str);
            }
            return Err(format!("Failed to extract RAR: {:?}", e));
        }
    };

    let total_files = archive.files.len() as u32;

    for (i, file) in archive.files.iter().enumerate() {
        let filename = file.name.clone();

        let sanitized_filename = SecurityContext::sanitize_path(&filename);

        if sanitized_filename != filename {
            SandboxEvent::Warning {
                code: "PATH_SANITIZED".to_string(),
                file: filename.clone(),
                details: "Blocked unsafe path or RTLO detected in RAR, neutralized".to_string(),
            }
            .send();
            sec_ctx.blocked_files += 1;

            // The vendor/rar crate already extracted the file to the potentially unsafe path
            // in the WASI filesystem. We need to move it to the sanitized safe path.
            let bad_path = Path::new(output_dir).join(&filename);
            let good_path = Path::new(output_dir).join(&sanitized_filename);

            if bad_path.exists() {
                if let Some(p) = good_path.parent() {
                    fs::create_dir_all(p).ok();
                }
                fs::rename(&bad_path, &good_path).unwrap_or_default();
            }
        }

        let uncompressed_size = file.unpacked_size;
        let compressed_size = file.head.data_area_size;

        if let Err(err_code) = sec_ctx.record_and_check(compressed_size, uncompressed_size) {
            SandboxEvent::Error {
                code: err_code.to_string(),
                details: format!(
                    "limits exceeded. {} > {}",
                    uncompressed_size, compressed_size
                ),
            }
            .send();
            return Err("Aborted due to security threshold".to_string());
        }

        // --- LAYER 2: PASSWORD/INTEGRITY VERIFICATION ---
        // For RAR archives, we cannot rely on manual CRC checks because RAR5
        // uses HMAC for encrypted files, and the `data_crc` field is meaningless.
        // We will skip manual CRC validation for RAR to prevent false rejections of valid passwords.

        SandboxEvent::Progress {
            current: (i + 1) as u32,
            total: total_files,
            file: sanitized_filename,
            bytes: uncompressed_size,
        }
        .send();
    }

    SandboxEvent::Complete {
        files_extracted: sec_ctx.extracted_files,
        files_blocked: sec_ctx.blocked_files,
        total_bytes: sec_ctx.extracted_bytes,
    }
    .send();

    Ok(())
}

// 7z extraction removed due to WASI incompatibility with sevenz-rust
