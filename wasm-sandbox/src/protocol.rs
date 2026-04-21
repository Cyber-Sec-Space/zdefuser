use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct HostLimit {
    pub max_ratio: u32,
    pub max_total_bytes: u64,
    pub max_files: u32,
}

#[derive(Debug, Deserialize)]
pub struct HostCommand {
    pub action: String,
    pub archive_path: String,
    pub output_dir: String,
    pub limits: HostLimit,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum SandboxEvent {
    #[serde(rename = "progress")]
    Progress {
        current: u32,
        total: u32,
        file: String,
        bytes: u64,
    },
    #[serde(rename = "warning")]
    Warning {
        code: String,
        file: String,
        details: String,
    },
    #[serde(rename = "complete")]
    Complete {
        files_extracted: u32,
        files_blocked: u32,
        total_bytes: u64,
    },
    #[serde(rename = "error")]
    Error {
        code: String,
        details: String,
    },
}

impl SandboxEvent {
    pub fn send(&self) {
        if let Ok(json) = serde_json::to_string(self) {
            println!("{}", json);
        }
    }
}
