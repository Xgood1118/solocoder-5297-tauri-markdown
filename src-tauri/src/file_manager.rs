use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use tokio::fs;
use walkdir::WalkDir;

use crate::error::AppResult;
use crate::models::{FileInfo, Document};

#[tauri::command]
pub async fn read_file(path: String) -> AppResult<Document> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err(crate::error::AppError::NotFound(path.to_string_lossy().to_string()));
    }

    let metadata = fs::metadata(path).await?;
    let size_bytes = metadata.len();

    let content = if size_bytes > 1024 * 1024 {
        fs::read_to_string(path).await?
    } else {
        fs::read_to_string(path).await?
    };

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();

    let created_at: DateTime<Utc> = metadata
        .created()
        .map(|t| t.into())
        .unwrap_or_else(|_| Utc::now());

    let updated_at: DateTime<Utc> = metadata
        .modified()
        .map(|t| t.into())
        .unwrap_or_else(|_| Utc::now());

    let is_encrypted = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "enc")
        .unwrap_or(false);

    Ok(Document {
        id: uuid::Uuid::new_v4(),
        path: path.to_string_lossy().to_string(),
        title,
        content,
        created_at,
        updated_at,
        is_dirty: false,
        is_encrypted,
        size_bytes,
    })
}

#[tauri::command]
pub async fn write_file(path: String, content: String) -> AppResult<bool> {
    let path = Path::new(&path);

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    let content_bytes = content.as_bytes();

    if content_bytes.len() > 1024 * 1024 {
        use tokio::io::AsyncWriteExt;
        let mut file = fs::File::create(path).await?;
        file.write_all(content_bytes).await?;
        file.flush().await?;
    } else {
        fs::write(path, content).await?;
    }

    Ok(true)
}

#[tauri::command]
pub async fn list_files(dir_path: String, recursive: Option<bool>) -> AppResult<Vec<FileInfo>> {
    let dir_path = Path::new(&dir_path);
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(crate::error::AppError::NotFound(dir_path.to_string_lossy().to_string()));
    }

    let mut files = Vec::new();
    let recursive = recursive.unwrap_or(false);

    if recursive {
        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                let name = entry
                    .file_name()
                    .to_str()
                    .unwrap_or("")
                    .to_string();

                let is_dir = metadata.is_dir();
                let size_bytes = metadata.len();

                let created_at: DateTime<Utc> = metadata
                    .created()
                    .map(|t| t.into())
                    .unwrap_or_else(|_| Utc::now());

                let modified_at: DateTime<Utc> = metadata
                    .modified()
                    .map(|t| t.into())
                    .unwrap_or_else(|_| Utc::now());

                files.push(FileInfo {
                    path: path.to_string_lossy().to_string(),
                    name,
                    is_dir,
                    size_bytes,
                    created_at,
                    modified_at,
                });
            }
        }
    } else {
        let mut entries = fs::read_dir(dir_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;

            let name = entry
                .file_name()
                .to_str()
                .unwrap_or("")
                .to_string();

            let is_dir = metadata.is_dir();
            let size_bytes = metadata.len();

            let created_at: DateTime<Utc> = metadata
                .created()
                .map(|t| t.into())
                .unwrap_or_else(|_| Utc::now());

            let modified_at: DateTime<Utc> = metadata
                .modified()
                .map(|t| t.into())
                .unwrap_or_else(|_| Utc::now());

            files.push(FileInfo {
                path: path.to_string_lossy().to_string(),
                name,
                is_dir,
                size_bytes,
                created_at,
                modified_at,
            });
        }
    }

    files.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(files)
}

#[tauri::command]
pub async fn create_file(path: String, is_dir: Option<bool>) -> AppResult<bool> {
    let path = Path::new(&path);
    let is_dir = is_dir.unwrap_or(false);

    if path.exists() {
        return Err(crate::error::AppError::InvalidArgument(
            "File already exists".to_string(),
        ));
    }

    if is_dir {
        fs::create_dir_all(path).await?;
    } else {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await?;
            }
        }
        fs::File::create(path).await?;
    }

    Ok(true)
}

#[tauri::command]
pub async fn delete_file(path: String) -> AppResult<bool> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err(crate::error::AppError::NotFound(path.to_string_lossy().to_string()));
    }

    if path.is_dir() {
        fs::remove_dir_all(path).await?;
    } else {
        fs::remove_file(path).await?;
    }

    Ok(true)
}

#[tauri::command]
pub async fn rename_file(old_path: String, new_path: String) -> AppResult<bool> {
    let old = Path::new(&old_path);
    let new = Path::new(&new_path);

    if !old.exists() {
        return Err(crate::error::AppError::NotFound(old_path));
    }

    if new.exists() {
        return Err(crate::error::AppError::InvalidArgument(
            "Target path already exists".to_string(),
        ));
    }

    fs::rename(old, new).await?;
    Ok(true)
}

#[tauri::command]
pub async fn create_directory(path: String) -> AppResult<bool> {
    let path = Path::new(&path);
    fs::create_dir_all(path).await?;
    Ok(true)
}

#[tauri::command]
pub async fn get_file_info(path: String) -> AppResult<FileInfo> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err(crate::error::AppError::NotFound(path.to_string_lossy().to_string()));
    }

    let metadata = fs::metadata(path).await?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let created_at: DateTime<Utc> = metadata
        .created()
        .map(|t| t.into())
        .unwrap_or_else(|_| Utc::now());

    let modified_at: DateTime<Utc> = metadata
        .modified()
        .map(|t| t.into())
        .unwrap_or_else(|_| Utc::now());

    Ok(FileInfo {
        path: path.to_string_lossy().to_string(),
        name,
        is_dir: metadata.is_dir(),
        size_bytes: metadata.len(),
        created_at,
        modified_at,
    })
}

#[tauri::command]
pub async fn save_image(
    base_dir: String,
    file_name: String,
    image_data: Vec<u8>,
) -> AppResult<String> {
    let assets_dir = PathBuf::from(&base_dir).join("assets");
    fs::create_dir_all(&assets_dir).await?;

    let file_stem = Path::new(&file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_string();

    let extension = Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    let mut final_name = format!("{}.{}", file_stem, extension);
    let mut counter = 1;

    while assets_dir.join(&final_name).exists() {
        final_name = format!("{}_{}.{}", file_stem, counter, extension);
        counter += 1;
    }

    let file_path = assets_dir.join(&final_name);
    fs::write(&file_path, &image_data).await?;

    Ok(format!("./assets/{}", final_name))
}

