use std::path::{Component, Path};

pub struct SecurityContext {
    pub max_ratio: u32,
    pub max_total_bytes: u64,
    pub max_files: u32,
    pub archive_size: u64,
    pub extracted_bytes: u64,
    pub extracted_files: u32,
    pub blocked_files: u32,
}

impl SecurityContext {
    pub fn new(max_ratio: u32, max_total_bytes: u64, max_files: u32, archive_size: u64) -> Self {
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

    /// Sanitize path to prevent Path Traversal and RTLO attacks.
    /// It strips dangerous components (like `..`, `/`, `C:\`) and returns a safe relative path.
    pub fn sanitize_path(filepath: &str) -> String {
        let path = Path::new(filepath);
        let mut sanitized = std::path::PathBuf::new();

        // Only keep Normal components (file and folder names)
        for component in path.components() {
            if let Component::Normal(c) = component {
                sanitized.push(c);
            }
        }

        let mut result = sanitized.to_string_lossy().into_owned();

        // --- RTLO and Dangerous Unicode Filtering ---
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

    /// Validate against zip bombs and limits
    pub fn record_and_check(
        &mut self,
        compressed_size: u64,
        uncompressed_size: u64,
    ) -> Result<(), &'static str> {
        self.extracted_files += 1;
        if self.extracted_files > self.max_files {
            return Err("TOO_MANY_FILES");
        }

        self.extracted_bytes += uncompressed_size;

        if self.extracted_bytes > self.max_total_bytes {
            return Err("TOTAL_SIZE_EXCEEDED");
        }

        // 1. Check individual file ratio if compressed size is known
        if uncompressed_size >= 1_048_576 && compressed_size > 0 {
            let ratio = uncompressed_size / compressed_size;
            if ratio > self.max_ratio as u64 {
                return Err("HIGH_RATIO_ZIP_BOMB");
            }
        }
        
        // 2. Check global ratio (Total Extracted vs Total Archive Size)
        // This catches solid archives (7z, tar.gz) where individual compressed size is unknown
        if self.archive_size > 0 {
            let global_ratio = self.extracted_bytes / self.archive_size;
            if global_ratio > self.max_ratio as u64 {
                return Err("HIGH_RATIO_ZIP_BOMB");
            }
        }

        Ok(())
    }
}
