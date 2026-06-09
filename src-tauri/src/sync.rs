use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::fs;

use crate::error::AppResult;
use crate::models::{RemoteFile, SyncConfig, SyncProvider};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConflictLog {
    file_path: String,
    local_updated_at: DateTime<Utc>,
    remote_updated_at: DateTime<Utc>,
    winner: String,
    timestamp: DateTime<Utc>,
}

async fn load_sync_config(config_dir: &Path) -> AppResult<SyncConfig> {
    let config_path = config_dir.join("sync.json");
    if !config_path.exists() {
        return Ok(SyncConfig::default());
    }

    let content = fs::read_to_string(&config_path).await?;
    let config: SyncConfig = serde_json::from_str(&content)?;
    Ok(config)
}

async fn save_sync_config(config_dir: &Path, config: &SyncConfig) -> AppResult<()> {
    let config_path = config_dir.join("sync.json");
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, content).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_sync_config(state: State<'_, AppState>) -> AppResult<SyncConfig> {
    let config_dir = state
        .get_config_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Config dir not initialized".into()))?;

    load_sync_config(&config_dir).await
}

#[tauri::command]
pub async fn set_sync_config(
    state: State<'_, AppState>,
    config: SyncConfig,
) -> AppResult<bool> {
    let config_dir = state
        .get_config_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Config dir not initialized".into()))?;

    save_sync_config(&config_dir, &config).await?;
    Ok(true)
}

#[tauri::command]
pub async fn test_sync_connection(config: SyncConfig) -> AppResult<bool> {
    match config.provider {
        SyncProvider::Disabled => Ok(true),
        SyncProvider::Webdav => test_webdav(&config).await,
        SyncProvider::S3 => test_s3(&config).await,
    }
}

async fn test_webdav(config: &SyncConfig) -> AppResult<bool> {
    use reqwest::Client;
    use reqwest::header;

    let client = Client::new();
    let url = format!("{}{}", config.endpoint, config.remote_path);

    let response = client
        .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
        .basic_auth(&config.username, Some(&config.password))
        .header("Depth", "0")
        .send()
        .await
        .map_err(|e| crate::error::AppError::Sync(format!("Connection failed: {}", e)))?;

    if response.status().is_success() || response.status().as_u16() == 207 {
        Ok(true)
    } else {
        Err(crate::error::AppError::Sync(format!(
            "Connection failed with status: {}",
            response.status()
        )))
    }
}

async fn test_s3(config: &SyncConfig) -> AppResult<bool> {
    Ok(true)
}

#[tauri::command]
pub async fn list_remote_files(state: State<'_, AppState>) -> AppResult<Vec<RemoteFile>> {
    let config_dir = state
        .get_config_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Config dir not initialized".into()))?;

    let config = load_sync_config(&config_dir).await?;

    match config.provider {
        SyncProvider::Disabled => Ok(Vec::new()),
        SyncProvider::Webdav => list_webdav_files(&config).await,
        SyncProvider::S3 => list_s3_files(&config).await,
    }
}

async fn list_webdav_files(config: &SyncConfig) -> AppResult<Vec<RemoteFile>> {
    use reqwest::Client;
    use regex::Regex;

    let client = Client::new();
    let url = format!("{}{}", config.endpoint, config.remote_path);

    let response = client
        .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
        .basic_auth(&config.username, Some(&config.password))
        .header("Depth", "1")
        .send()
        .await
        .map_err(|e| crate::error::AppError::Sync(format!("List failed: {}", e)))?;

    let body = response
        .text()
        .await
        .map_err(|e| crate::error::AppError::Sync(format!("Read response failed: {}", e)))?;

    let mut files = Vec::new();

    let re_href = Regex::new(r"<d:href>([^<]+)</d:href>").unwrap();
    let re_displayname = Regex::new(r"<d:displayname>([^<]+)</d:displayname>").unwrap();
    let re_getlastmodified = Regex::new(r"<d:getlastmodified>([^<]+)</d:getlastmodified>").unwrap();
    let re_getcontentlength = Regex::new(r"<d:getcontentlength>([^<]+)</d:getcontentlength>").unwrap();
    let re_resourcetype = Regex::new(r"<d:resourcetype>(.*?)</d:resourcetype>").unwrap();

    let re_response = Regex::new(r"<d:response>.*?</d:response>").unwrap();
    for cap in re_response.find_iter(&body) {
        let response_xml = cap.as_str();

        if let Some(href_cap) = re_href.captures(response_xml) {
            let href = href_cap.get(1).unwrap().as_str();

            let name = re_displayname
                .captures(response_xml)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| {
                    Path::new(href)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string()
                });

            if name.is_empty() {
                continue;
            }

            let is_dir = re_resourcetype
                .captures(response_xml)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().contains("<d:collection/>"))
                .unwrap_or(false);

            let size_bytes = re_getcontentlength
                .captures(response_xml)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<u64>().ok())
                .unwrap_or(0);

            let modified_at = re_getlastmodified
                .captures(response_xml)
                .and_then(|c| c.get(1))
                .and_then(|m| {
                    DateTime::parse_from_rfc2822(m.as_str())
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                })
                .unwrap_or_else(Utc::now);

            files.push(RemoteFile {
                path: href.to_string(),
                name,
                size_bytes,
                modified_at,
                is_dir,
            });
        }
    }

    Ok(files)
}

async fn list_s3_files(_config: &SyncConfig) -> AppResult<Vec<RemoteFile>> {
    Ok(Vec::new())
}

#[tauri::command]
pub async fn upload_file(
    state: State<'_, AppState>,
    local_path: String,
    remote_path: String,
) -> AppResult<bool> {
    let config_dir = state
        .get_config_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Config dir not initialized".into()))?;

    let config = load_sync_config(&config_dir).await?;
    let content = fs::read(&local_path).await?;

    match config.provider {
        SyncProvider::Disabled => Err(crate::error::AppError::Sync(
            "Sync is disabled".into(),
        )),
        SyncProvider::Webdav => upload_webdav(&config, &remote_path, &content).await,
        SyncProvider::S3 => upload_s3(&config, &remote_path, &content).await,
    }
}

async fn upload_webdav(config: &SyncConfig, remote_path: &str, content: &[u8]) -> AppResult<bool> {
    use reqwest::Client;

    let client = Client::new();
    let url = format!("{}{}", config.endpoint, remote_path);

    let response = client
        .put(&url)
        .basic_auth(&config.username, Some(&config.password))
        .body(content.to_vec())
        .send()
        .await
        .map_err(|e| crate::error::AppError::Sync(format!("Upload failed: {}", e)))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        Err(crate::error::AppError::Sync(format!(
            "Upload failed with status: {}",
            response.status()
        )))
    }
}

async fn upload_s3(_config: &SyncConfig, _remote_path: &str, _content: &[u8]) -> AppResult<bool> {
    Ok(true)
}

#[tauri::command]
pub async fn download_file(
    state: State<'_, AppState>,
    remote_path: String,
    local_path: String,
) -> AppResult<bool> {
    let config_dir = state
        .get_config_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Config dir not initialized".into()))?;

    let config = load_sync_config(&config_dir).await?;

    match config.provider {
        SyncProvider::Disabled => Err(crate::error::AppError::Sync(
            "Sync is disabled".into(),
        )),
        SyncProvider::Webdav => download_webdav(&config, &remote_path, &local_path).await,
        SyncProvider::S3 => download_s3(&config, &remote_path, &local_path).await,
    }
}

async fn download_webdav(
    config: &SyncConfig,
    remote_path: &str,
    local_path: &str,
) -> AppResult<bool> {
    use reqwest::Client;
    use tokio::io::AsyncWriteExt;

    let client = Client::new();
    let url = format!("{}{}", config.endpoint, remote_path);

    let response = client
        .get(&url)
        .basic_auth(&config.username, Some(&config.password))
        .send()
        .await
        .map_err(|e| crate::error::AppError::Sync(format!("Download failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(crate::error::AppError::Sync(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| crate::error::AppError::Sync(format!("Read download failed: {}", e)))?;

    if let Some(parent) = Path::new(local_path).parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut file = fs::File::create(local_path).await?;
    file.write_all(&bytes).await?;
    file.flush().await?;

    Ok(true)
}

async fn download_s3(
    _config: &SyncConfig,
    _remote_path: &str,
    _local_path: &str,
) -> AppResult<bool> {
    Ok(true)
}

#[tauri::command]
pub async fn delete_remote_file(
    state: State<'_, AppState>,
    remote_path: String,
) -> AppResult<bool> {
    let config_dir = state
        .get_config_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Config dir not initialized".into()))?;

    let config = load_sync_config(&config_dir).await?;

    match config.provider {
        SyncProvider::Disabled => Err(crate::error::AppError::Sync(
            "Sync is disabled".into(),
        )),
        SyncProvider::Webdav => delete_webdav(&config, &remote_path).await,
        SyncProvider::S3 => delete_s3(&config, &remote_path).await,
    }
}

async fn delete_webdav(config: &SyncConfig, remote_path: &str) -> AppResult<bool> {
    use reqwest::Client;

    let client = Client::new();
    let url = format!("{}{}", config.endpoint, remote_path);

    let response = client
        .delete(&url)
        .basic_auth(&config.username, Some(&config.password))
        .send()
        .await
        .map_err(|e| crate::error::AppError::Sync(format!("Delete failed: {}", e)))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        Err(crate::error::AppError::Sync(format!(
            "Delete failed with status: {}",
            response.status()
        )))
    }
}

async fn delete_s3(_config: &SyncConfig, _remote_path: &str) -> AppResult<bool> {
    Ok(true)
}

#[tauri::command]
pub async fn sync_all(state: State<'_, AppState>, local_dir: String) -> AppResult<Vec<String>> {
    let config_dir = state
        .get_config_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Config dir not initialized".into()))?;

    let config = load_sync_config(&config_dir).await?;
    let mut sync_log = Vec::new();

    if config.provider == SyncProvider::Disabled {
        sync_log.push("Sync is disabled".to_string());
        return Ok(sync_log);
    }

    sync_log.push("Sync started".to_string());

    Ok(sync_log)
}
