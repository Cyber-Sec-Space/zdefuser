use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct SandboxEnv {
    temp_dir: TempDir,
}

impl SandboxEnv {
    pub fn new() -> Result<Self, String> {
        let temp_dir = tempfile::Builder::new()
            .prefix("zdefuser_")
            .tempdir()
            .map_err(|e| format!("Failed to create tempdir: {}", e))?;

        Ok(Self { temp_dir })
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn release_dir(&self) -> PathBuf {
        let release = self.temp_dir.path().join("output");
        fs::create_dir_all(&release).ok();
        release
    }

    pub fn copy_input(&self, source_path: &str) -> Result<String, String> {
        let path = Path::new(source_path);
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("input.archive");
        let target = self.temp_dir.path().join(file_name);

        fs::copy(source_path, &target).map_err(|e| format!("Failed to copy input: {}", e))?;

        Ok(format!("/sandbox/{}", file_name))
    }
}
