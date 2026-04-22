use std::path::{Component, Path};

pub struct SecurityContext {
    pub max_ratio: u32,
    pub max_total_bytes: u64,
    pub extracted_bytes: u64,
    pub extracted_files: u32,
    pub blocked_files: u32,
    pub max_files: u32,
}

impl SecurityContext {
    pub fn new(max_ratio: u32, max_total_bytes: u64, max_files: u32) -> Self {
        Self {
            max_ratio,
            max_total_bytes,
            extracted_bytes: 0,
            extracted_files: 0,
            blocked_files: 0,
            max_files,
        }
    }

    /// Check for Path Traversal in the extracted filename
    pub fn is_safe_path(filepath: &str) -> bool {
        let path = Path::new(filepath);
        if path.is_absolute() {
            return false;
        }

        // Must not contain any ParentDir ("..") components
        for component in path.components() {
            match component {
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => return false,
                _ => {}
            }
        }

        // Further deny if it includes backslashes or unexpected traversal tricks (though Path normalizes most)
        if filepath.contains("..") || filepath.starts_with('/') || filepath.starts_with('\\') {
            return false;
        }

        // --- RTLO and Dangerous Unicode Filtering ---
        // \u{202E} is Right-to-Left Override, often used to disguise extensions (e.g. txt.exe)
        // \u{200F} is Right-to-Left Mark
        // \u{202A} to \u{202F} are mostly bidirectional formatting codes
        for c in filepath.chars() {
            if c == '\u{202E}' || c == '\u{200F}' || (c >= '\u{202A}' && c <= '\u{202F}') {
                return false; // Dangerous spoofing attempt
            }
        }

        true
    }

    /// Validate against zip bombs and limits 
    pub fn record_and_check(&mut self, compressed_size: u64, uncompressed_size: u64) -> Result<(), &'static str> {
        self.extracted_files += 1;
        if self.extracted_files > self.max_files {
            return Err("TOO_MANY_FILES");
        }

        self.extracted_bytes += uncompressed_size;

        if self.extracted_bytes > self.max_total_bytes {
            return Err("TOTAL_SIZE_EXCEEDED");
        }

        // Only check ratio if uncompressed size is decently big (e.g., >= 1 MB)
        if uncompressed_size >= 1_048_576 && compressed_size > 0 {
            let ratio = uncompressed_size / compressed_size;
            if ratio > self.max_ratio as u64 {
                return Err("HIGH_RATIO_ZIP_BOMB");
            }
        }

        Ok(())
    }
}
