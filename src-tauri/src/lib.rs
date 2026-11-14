pub mod agents;
pub mod db;
pub mod ipc;
pub mod sandbox;

use db::repository::DatabaseRepository;
use ipc::commands::{
    export_signed_wallet, get_api_key_status, get_curriculum, request_socratic_hint, save_api_key,
    submit_code, toggle_theme, update_student_name, verify_signed_wallet, AppState,
};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            std::fs::create_dir_all(&data_dir)?;
            let db_filename = std::env::var("LOGICKIOSK_DB_FILENAME")
                .unwrap_or_else(|_| "logickiosk_state.sqlite".to_string());
            let db_path = data_dir.join(db_filename);
            let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy().replace('\\', "/"));

            let repo = tauri::async_runtime::block_on(DatabaseRepository::connect_and_migrate(&db_url))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            app.manage(AppState {
                repo: Arc::new(repo),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_curriculum,
            submit_code,
            request_socratic_hint,
            toggle_theme,
            update_student_name,
            save_api_key,
            get_api_key_status,
            export_signed_wallet,
            verify_signed_wallet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running LogicKiosk Tauri application");
}
