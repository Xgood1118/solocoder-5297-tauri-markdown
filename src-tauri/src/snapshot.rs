use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use tauri::State;
use tokio::fs;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::Snapshot;
use crate::state::AppState;

const MAX_SNAPSHOTS: usize = 50;

fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn get_snapshot_key(file_path: &str) -> String {
    let hash = content_hash(file_path);
    hash.chars().take(16).collect()
}

async fn load_snapshots_meta(snapshot_dir: &Path, file_key: &str) -> AppResult<Vec<Snapshot>> {
    let meta_path = snapshot_dir.join(format!("{}.json", file_key));
    if !meta_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&meta_path).await?;
    let snapshots: Vec<Snapshot> = serde_json::from_str(&content)?;
    Ok(snapshots)
}

async fn save_snapshots_meta(
    snapshot_dir: &Path,
    file_key: &str,
    snapshots: &[Snapshot],
) -> AppResult<()> {
    let meta_path = snapshot_dir.join(format!("{}.json", file_key));
    let content = serde_json::to_string_pretty(snapshots)?;
    fs::write(&meta_path, content).await?;
    Ok(())
}

#[tauri::command]
pub async fn create_snapshot(
    state: State<'_, AppState>,
    file_path: String,
    content: String,
) -> AppResult<Snapshot> {
    let snapshot_dir = state
        .get_snapshot_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Snapshot dir not initialized".into()))?;

    let file_key = get_snapshot_key(&file_path);
    let hash = content_hash(&content);

    let mut snapshots = load_snapshots_meta(&snapshot_dir, &file_key).await?;

    if snapshots.iter().any(|s| s.content_hash == hash) {
        return Err(crate::error::AppError::InvalidArgument(
            "Duplicate snapshot".to_string(),
        ));
    }

    let snapshot = Snapshot {
        id: Uuid::new_v4(),
        file_path: file_path.clone(),
        content_hash: hash.clone(),
        content: content.clone(),
        created_at: Utc::now(),
        size_bytes: content.len() as u64,
    };

    let content_dir = snapshot_dir.join("content");
    fs::create_dir_all(&content_dir).await?;
    let content_file = content_dir.join(format!("{}.md", hash));
    if !content_file.exists() {
        fs::write(&content_file, &content).await?;
    }

    snapshots.push(snapshot.clone());

    if snapshots.len() > MAX_SNAPSHOTS {
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let to_remove: Vec<Snapshot> = snapshots.drain(MAX_SNAPSHOTS..).collect();
        for s in to_remove {
            let content_file = content_dir.join(format!("{}.md", s.content_hash));
            if content_file.exists() {
                let _ = fs::remove_file(&content_file).await;
            }
        }
    }

    save_snapshots_meta(&snapshot_dir, &file_key, &snapshots).await?;

    Ok(snapshot)
}

#[tauri::command]
pub async fn list_snapshots(
    state: State<'_, AppState>,
    file_path: String,
) -> AppResult<Vec<Snapshot>> {
    let snapshot_dir = state
        .get_snapshot_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Snapshot dir not initialized".into()))?;

    let file_key = get_snapshot_key(&file_path);
    let mut snapshots = load_snapshots_meta(&snapshot_dir, &file_key).await?;

    snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    for s in snapshots.iter_mut() {
        s.content = String::new();
    }

    Ok(snapshots)
}

#[tauri::command]
pub async fn restore_snapshot(
    state: State<'_, AppState>,
    snapshot_id: String,
    file_path: String,
) -> AppResult<String> {
    let snapshot_dir = state
        .get_snapshot_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Snapshot dir not initialized".into()))?;

    let file_key = get_snapshot_key(&file_path);
    let snapshots = load_snapshots_meta(&snapshot_dir, &file_key).await?;

    let snapshot = snapshots
        .iter()
        .find(|s| s.id.to_string() == snapshot_id)
        .ok_or_else(|| crate::error::AppError::NotFound("Snapshot not found".into()))?;

    let content_dir = snapshot_dir.join("content");
    let content_file = content_dir.join(format!("{}.md", snapshot.content_hash));

    let content = fs::read_to_string(&content_file).await?;
    Ok(content)
}

#[tauri::command]
pub async fn delete_snapshot(
    state: State<'_, AppState>,
    snapshot_id: String,
    file_path: String,
) -> AppResult<bool> {
    let snapshot_dir = state
        .get_snapshot_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Snapshot dir not initialized".into()))?;

    let file_key = get_snapshot_key(&file_path);
    let mut snapshots = load_snapshots_meta(&snapshot_dir, &file_key).await?;

    if let Some(pos) = snapshots.iter().position(|s| s.id.to_string() == snapshot_id) {
        let removed = snapshots.remove(pos);
        let content_dir = snapshot_dir.join("content");
        let content_file = content_dir.join(format!("{}.md", removed.content_hash));

        let hash_used_elsewhere = snapshots
            .iter()
            .any(|s| s.content_hash == removed.content_hash);

        if !hash_used_elsewhere && content_file.exists() {
            let _ = fs::remove_file(&content_file).await;
        }

        save_snapshots_meta(&snapshot_dir, &file_key, &snapshots).await?;
        Ok(true)
    } else {
        Err(crate::error::AppError::NotFound(
            "Snapshot not found".into(),
        ))
    }
}
