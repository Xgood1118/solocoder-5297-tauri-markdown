use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

pub mod error;
pub mod models;
pub mod file_manager;
pub mod snapshot;
pub mod search;
pub mod encryption;
pub mod sync;
pub mod export;
pub mod plugin;
pub mod state;

pub use error::AppError;
pub use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            docs: Arc::new(Mutex::new(std::collections::HashMap::new())),
            config_dir: Arc::new(Mutex::new(None)),
            snapshot_dir: Arc::new(Mutex::new(None)),
            plugin_dir: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            file_manager::read_file,
            file_manager::write_file,
            file_manager::list_files,
            file_manager::create_file,
            file_manager::delete_file,
            file_manager::rename_file,
            file_manager::create_directory,
            file_manager::get_file_info,
            file_manager::save_image,
            snapshot::create_snapshot,
            snapshot::list_snapshots,
            snapshot::restore_snapshot,
            snapshot::delete_snapshot,
            search::search_files,
            search::search_content,
            encryption::encrypt_file,
            encryption::decrypt_file,
            encryption::set_password,
            sync::list_remote_files,
            sync::upload_file,
            sync::download_file,
            sync::delete_remote_file,
            sync::sync_all,
            sync::get_sync_config,
            sync::set_sync_config,
            sync::test_sync_connection,
            export::export_html,
            export::export_pdf,
            export::export_docx,
            plugin::list_plugins,
            plugin::load_plugin,
            plugin::unload_plugin,
            plugin::run_plugin_command,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<AppState>();
                state.init_dirs().await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
