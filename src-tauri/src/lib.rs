pub mod sandbox;
pub mod wasm;
pub mod security;
pub mod release;
pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::SandboxState(std::sync::Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            commands::analyze_archive,
            commands::release_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
