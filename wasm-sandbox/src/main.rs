mod protocol;
mod security;
mod decompress;

use protocol::{HostCommand, SandboxEvent};
use std::io::{self, BufRead};

fn main() {
    // 1. Read the command from standard input
    let stdin = io::stdin();
    let mut input = String::new();
    if stdin.lock().read_line(&mut input).is_err() || input.trim().is_empty() {
        SandboxEvent::Error {
            code: "NO_INPUT".to_string(),
            details: "Failed to read HostCommand from stdin".to_string(),
        }.send();
        return;
    }

    // 2. Deserialize HostCommand
    let cmd: HostCommand = match serde_json::from_str(&input) {
        Ok(c) => c,
        Err(e) => {
            SandboxEvent::Error {
                code: "INVALID_JSON".to_string(),
                details: e.to_string(),
            }.send();
            return;
        }
    };

    // 3. Determine archive type and execute 
    if cmd.action == "extract" {
        let result = if cmd.archive_path.ends_with(".zip") {
            decompress::extract_zip(&cmd.archive_path, &cmd.output_dir, cmd.password.as_deref(), &cmd.limits)
        } else if cmd.archive_path.ends_with(".rar") {
            decompress::extract_rar(&cmd.archive_path, &cmd.output_dir, cmd.password.as_deref(), &cmd.limits)
        } else if cmd.archive_path.ends_with(".tar") {
            decompress::extract_tar(&cmd.archive_path, &cmd.output_dir, false, &cmd.limits)
        } else if cmd.archive_path.ends_with(".tar.gz") || cmd.archive_path.ends_with(".tgz") {
            decompress::extract_tar(&cmd.archive_path, &cmd.output_dir, true, &cmd.limits)
        } else {
            Err("Unsupported format".to_string())
        };

        if let Err(e) = result {
            SandboxEvent::Error {
                code: "EXTRACT_FAILED".to_string(),
                details: e,
            }.send();
        }
    } else {
        SandboxEvent::Error {
            code: "UNKNOWN_ACTION".to_string(),
            details: format!("Action `{}` not supported", cmd.action),
        }.send();
    }
}
