use tauri::{AppHandle, Emitter};
use serde_json::json;
use crate::sandbox::SandboxEnv;
use crate::wasm::run_wasm_sandbox;
use crate::security;
use crate::release;

// Shared state for the current sandbox environment
pub struct SandboxState(pub std::sync::Mutex<Option<std::sync::Arc<SandboxEnv>>>);

#[tauri::command]
pub async fn analyze_archive(app: AppHandle, state: tauri::State<'_, SandboxState>, archive_path: String) -> Result<(), String> {
    let env = std::sync::Arc::new(SandboxEnv::new().map_err(|e| e.to_string())?);
    
    // Copy the archive into the sandbox, preserving the filename
    let sandbox_file_path = env.copy_input(&archive_path)?;

    // We build the json command
    let limits = json!({
        "max_ratio": 100,
        "max_total_bytes": 100 * 1024 * 1024 * 1024 as u64, // 100GB
        "max_files": 500_000 // For huge dataset/node_modules zip bombs
    });
    
    let cmd = json!({
        "action": "extract",
        "archive_path": sandbox_file_path,
        "output_dir": "/sandbox/output/",
        "limits": limits
    }).to_string();

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // Run wasm execution in a blocking thread to not block async runtime
    let env_thread = env.clone();
    
    let _ = tokio::task::spawn_blocking(move || {
        let _ = run_wasm_sandbox(&env_thread, cmd, tx);
    });

    // Listen to wasm messages and forward them to frontend
    while let Some(msg) = rx.recv().await {
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&msg) {
            app.emit("sandbox_event", json_val).unwrap_or(());
        }
    }

    // Now Layer 2 check
    let release_dir = env.release_dir();
    if let Err(e) = security::validate_extracted_dir(&release_dir, 2 * 1024 * 1024 * 1024) {
        app.emit("sandbox_event", json!({"type": "error", "code": "LAYER_2_FAIL", "details": e})).unwrap_or(());
        return Err(e);
    }

    // Save successful environment into state so `release_files` can use it
    let mut st = state.0.lock().unwrap();
    *st = Some(env.clone());
    
    Ok(())
}

#[tauri::command]
pub async fn release_files(state: tauri::State<'_, SandboxState>, target_dir: String) -> Result<(), String> {
    let mut st = state.0.lock().unwrap();
    if let Some(env) = st.take() {
        let release_dir = env.release_dir();
        let target_path = std::path::Path::new(&target_dir);
        
        release::move_and_strip_permissions(&release_dir, target_path)?;
        Ok(())
    } else {
        Err("No active sandbox session to release from".to_string())
    }
}
