use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::fs;

use crate::error::AppResult;
use crate::models::{PluginInfo, PluginType};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginType,
    pub main: String,
}

#[tauri::command]
pub async fn list_plugins(state: State<'_, AppState>) -> AppResult<Vec<PluginInfo>> {
    let plugin_dir = state
        .get_plugin_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Plugin dir not initialized".into()))?;

    if !plugin_dir.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();

    let mut entries = fs::read_dir(&plugin_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                if let Ok(content) = fs::read_to_string(&manifest_path).await {
                    if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&content) {
                        plugins.push(PluginInfo {
                            id: manifest.id,
                            name: manifest.name,
                            version: manifest.version,
                            description: manifest.description,
                            author: manifest.author,
                            plugin_type: manifest.plugin_type,
                            enabled: true,
                        });
                    }
                }
            }
        }
    }

    Ok(plugins)
}

#[tauri::command]
pub async fn load_plugin(
    state: State<'_, AppState>,
    plugin_id: String,
) -> AppResult<String> {
    let plugin_dir = state
        .get_plugin_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Plugin dir not initialized".into()))?;

    let plugin_path = plugin_dir.join(&plugin_id);
    let manifest_path = plugin_path.join("manifest.json");

    if !manifest_path.exists() {
        return Err(crate::error::AppError::NotFound(format!(
            "Plugin {} not found",
            plugin_id
        )));
    }

    let manifest_content = fs::read_to_string(&manifest_path).await?;
    let manifest: PluginManifest = serde_json::from_str(&manifest_content)?;

    let main_file = plugin_path.join(&manifest.main);
    if !main_file.exists() {
        return Err(crate::error::AppError::NotFound(format!(
            "Plugin main file {} not found",
            manifest.main
        )));
    }

    let plugin_code = fs::read_to_string(&main_file).await?;
    Ok(plugin_code)
}

#[tauri::command]
pub async fn unload_plugin(
    state: State<'_, AppState>,
    plugin_id: String,
) -> AppResult<bool> {
    let _ = state;
    let _ = plugin_id;
    Ok(true)
}

#[tauri::command]
pub async fn run_plugin_command(
    state: State<'_, AppState>,
    plugin_id: String,
    command: String,
    args: serde_json::Value,
) -> AppResult<serde_json::Value> {
    let _ = state;
    let _ = plugin_id;
    let _ = command;
    let _ = args;

    Ok(serde_json::json!({ "result": "success" }))
}
