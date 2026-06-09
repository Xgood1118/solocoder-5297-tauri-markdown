use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::Document;

#[derive(Clone)]
pub struct AppState {
    pub docs: Arc<Mutex<HashMap<String, Document>>>,
    pub config_dir: Arc<Mutex<Option<PathBuf>>>,
    pub snapshot_dir: Arc<Mutex<Option<PathBuf>>>,
    pub plugin_dir: Arc<Mutex<Option<PathBuf>>>,
}

impl AppState {
    pub async fn init_dirs(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config = config_dir.join("markdown-editor");
            let _ = std::fs::create_dir_all(&app_config);

            let snapshot_dir = app_config.join("snapshots");
            let _ = std::fs::create_dir_all(&snapshot_dir);

            let plugin_dir = app_config.join("plugins");
            let _ = std::fs::create_dir_all(&plugin_dir);

            *self.config_dir.lock().await = Some(app_config.clone());
            *self.snapshot_dir.lock().await = Some(snapshot_dir);
            *self.plugin_dir.lock().await = Some(plugin_dir);
        }
    }

    pub async fn get_config_dir(&self) -> Option<PathBuf> {
        self.config_dir.lock().await.clone()
    }

    pub async fn get_snapshot_dir(&self) -> Option<PathBuf> {
        self.snapshot_dir.lock().await.clone()
    }

    pub async fn get_plugin_dir(&self) -> Option<PathBuf> {
        self.plugin_dir.lock().await.clone()
    }
}
