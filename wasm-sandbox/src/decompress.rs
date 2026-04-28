use std::fs::{self, File};
use std::io::{copy, Read};
use std::path::Path;

use flate2::read::GzDecoder;
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
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid Zip: {}", e))?;
    let mut sec_ctx =
        SecurityContext::new(limits.max_ratio, limits.max_total_bytes, limits.max_files);

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

        if !SecurityContext::is_safe_path(filename.to_str().unwrap_or("")) {
            SandboxEvent::Warning {
                code: "PATH_TRAVERSAL".to_string(),
                file: filename.display().to_string(),
                details: "Blocked due to unsafe path traversal".to_string(),
            }
            .send();
            sec_ctx.blocked_files += 1;
            continue;
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

        let out_path = out_dir.join(&filename);
        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(p) = out_path.parent()
                && !p.exists() {
                    fs::create_dir_all(p).ok();
                }
            let mut outfile =
                File::create(&out_path).map_err(|e| format!("Write failed: {}", e))?;
            copy(&mut file.take(uncompressed_size), &mut outfile).map_err(|e| format!("Extract failed: {}", e))?;

            SandboxEvent::Progress {
                current: (i + 1) as u32,
                total: total_files as u32,
                file: filename.display().to_string(),
                bytes: uncompressed_size,
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
    let mut sec_ctx =
        SecurityContext::new(limits.max_ratio, limits.max_total_bytes, limits.max_files);
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

        if !SecurityContext::is_safe_path(filename.to_str().unwrap_or("")) {
            SandboxEvent::Warning {
                code: "PATH_TRAVERSAL".to_string(),
                file: filename.display().to_string(),
                details: "Blocked unsafe path".to_string(),
            }
            .send();
            sec_ctx.blocked_files += 1;
            continue;
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

        count += 1;
        let out_path = out_dir.join(&filename);
        if entry_type.is_dir() {
            fs::create_dir_all(&out_path).ok();
        } else if entry_type.is_file() {
            if let Some(p) = out_path.parent()
                && !p.exists() {
                    fs::create_dir_all(p).ok();
                }
            let mut outfile =
                File::create(&out_path).map_err(|e| format!("Write failed: {}", e))?;
            copy(&mut entry.take(uncompressed_size), &mut outfile).map_err(|e| format!("Extract failed: {}", e))?;
            SandboxEvent::Progress {
                current: count,
                total: 0, // Tar streams unknown total
                file: filename.display().to_string(),
                bytes: uncompressed_size,
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
    let pwd = password.unwrap_or("");
    let mut sec_ctx =
        SecurityContext::new(limits.max_ratio, limits.max_total_bytes, limits.max_files);

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

        // --- LAYER 1: UNICODE SPOOFING (RTLO) & PATH TRAVERSAL VERIFICATION ---
        // Since `extract_all` already dumped the files into the WASI sandbox,
        // we must ABORT the entire operation if ANY file contains dangerous paths.
        if !SecurityContext::is_safe_path(&filename) {
            SandboxEvent::Error {
                code: "PATH_TRAVERSAL".to_string(),
                details: format!("Blocked unsafe path or RTLO detected in RAR: {}", filename),
            }
            .send();
            return Err("Aborted due to security boundary violation".to_string());
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
        // Since `rar-rs` blindy decrypts even with bad passwords (garbling the data),
        // we must enforce validation by comparing the checksum of the resulting stream.
        if file.unpacked_size > 0 && !file.flags.directory {
            use std::io::Read;
            let out_path = std::path::Path::new(&out_dir).join(&filename);
            if let Ok(mut extracted_file) = std::fs::File::open(&out_path) {
                let mut hasher = crc32fast::Hasher::new();
                let mut buf = vec![0u8; 1024 * 8];
                while let Ok(n) = extracted_file.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buf[..n]);
                }
                let checksum = hasher.finalize();

                // RAR format uses CRC, when encryption yields garbage, this will inevitably fail
                if checksum != file.data_crc {
                    std::fs::remove_file(&out_path).ok(); // Clean up corrupted chunk

                    let err_str = "Password required or invalid for encrypted archive. Data verification failed.".to_string();
                    SandboxEvent::Error {
                        code: "PASSWORD_REQUIRED".to_string(),
                        details: err_str.clone(),
                    }
                    .send();

                    return Err(err_str);
                }
            }
        }

        SandboxEvent::Progress {
            current: (i + 1) as u32,
            total: total_files,
            file: filename,
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

pub fn extract_7z(
    archive_path: &str,
    output_dir: &str,
    password: Option<&str>,
    limits: &HostLimit,
) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| format!("Failed to open 7z: {}", e))?;
    let mut sec_ctx =
        SecurityContext::new(limits.max_ratio, limits.max_total_bytes, limits.max_files);
    let out_dir = Path::new(output_dir);

    let pwd = match password {
        Some(p) => sevenz_rust::Password::from(p),
        None => sevenz_rust::Password::empty(),
    };

    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut archive = sevenz_rust::SevenZReader::new(file, len, pwd)
        .map_err(|e| format!("Invalid 7z or Password required: {}", e))?;

    let mut count = 0;
    archive
        .for_each_entries(|entry, reader| {
            let filename_str = entry.name();
            if filename_str.is_empty() {
                return Ok(true);
            }

            if !SecurityContext::is_safe_path(filename_str) {
                SandboxEvent::Warning {
                    code: "PATH_TRAVERSAL".to_string(),
                    file: filename_str.to_string(),
                    details: "Blocked unsafe path".to_string(),
                }
                .send();
                sec_ctx.blocked_files += 1;
                return Ok(true);
            }

            // Note: 7z format inherently supports anti-item attributes, but symlink support
            // in standard 7z crates usually surfaces as regular files or is just ignored.
            // But we can check if it's explicitly marked as anti-item / symlink if supported.
            // As a pure stream extractor, we just extract files and directories.

            let uncompressed_size = entry.size();
            
            if entry.has_stream() {
                // 7z uses solid compression so individual compressed sizes are tricky,
                // bypass ratio limit by asserting ratio of 1, rely on max_total_bytes & Fuel.
                if let Err(err_code) = sec_ctx.record_and_check(uncompressed_size.max(1), uncompressed_size) {
                    SandboxEvent::Error {
                        code: err_code.to_string(),
                        details: "limit exceeded".to_string(),
                    }
                    .send();
                    return Err(sevenz_rust::Error::other("Aborted due to security threshold"));
                }
            }

            count += 1;
            let out_path = out_dir.join(filename_str);
            
            if entry.is_directory() {
                fs::create_dir_all(&out_path).ok();
            } else if entry.has_stream() {
                if let Some(p) = out_path.parent()
                    && !p.exists() {
                        fs::create_dir_all(p).ok();
                    }
                if let Ok(mut outfile) = File::create(&out_path) {
                    let _ = copy(&mut reader.take(uncompressed_size), &mut outfile);
                }
                SandboxEvent::Progress {
                    current: count,
                    total: 0, // Solid block doesn't give total easily during iterator
                    file: filename_str.to_string(),
                    bytes: uncompressed_size,
                }
                .send();
            }

            Ok(true)
        })
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("Aborted due to security threshold") {
                msg
            } else {
                format!("7z extraction error: {}", msg)
            }
        })?;

    SandboxEvent::Complete {
        files_extracted: sec_ctx.extracted_files,
        files_blocked: sec_ctx.blocked_files,
        total_bytes: sec_ctx.extracted_bytes,
    }
    .send();

    Ok(())
}
