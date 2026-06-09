use std::path::Path;
use walkdir::WalkDir;

use crate::error::AppResult;
use crate::models::SearchResult;

#[tauri::command]
pub async fn search_files(dir_path: String, pattern: String) -> AppResult<Vec<String>> {
    let dir_path = Path::new(&dir_path);
    if !dir_path.exists() {
        return Err(crate::error::AppError::NotFound(dir_path.to_string_lossy().to_string()));
    }

    let mut results = Vec::new();
    let pattern_lower = pattern.to_lowercase();

    for entry in WalkDir::new(dir_path).max_depth(10).into_iter().flatten() {
        let path = entry.path();
        if path.is_file() && is_markdown_file(path) {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.to_lowercase().contains(&pattern_lower) {
                    results.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn search_content(
    dir_path: String,
    pattern: String,
    case_sensitive: Option<bool>,
) -> AppResult<Vec<SearchResult>> {
    let dir_path = Path::new(&dir_path);
    if !dir_path.exists() {
        return Err(crate::error::AppError::NotFound(dir_path.to_string_lossy().to_string()));
    }

    let case_sensitive = case_sensitive.unwrap_or(false);
    let mut results = Vec::new();

    for entry in WalkDir::new(dir_path).max_depth(10).into_iter().flatten() {
        let path = entry.path();
        if !path.is_file() || !is_markdown_file(path) {
            continue;
        }

        let path_str = path.to_string_lossy().to_string();

        if let Ok(content) = std::fs::read_to_string(path) {
            search_in_content(&path_str, &content, &pattern, case_sensitive, &mut results);
        }

        if results.len() >= 500 {
            break;
        }
    }

    Ok(results)
}

fn search_in_content(
    file_path: &str,
    content: &str,
    pattern: &str,
    case_sensitive: bool,
    results: &mut Vec<SearchResult>,
) {
    let search_pattern = if case_sensitive {
        pattern.to_string()
    } else {
        pattern.to_lowercase()
    };

    for (line_num, line) in content.lines().enumerate() {
        let line_to_search = if case_sensitive {
            line.to_string()
        } else {
            line.to_lowercase()
        };

        let mut start = 0;
        while let Some(pos) = line_to_search[start..].find(&search_pattern) {
            let col_start = start + pos;
            let col_end = col_start + pattern.len();

            results.push(SearchResult {
                file_path: file_path.to_string(),
                line_number: line_num + 1,
                column_start: col_start + 1,
                column_end: col_end + 1,
                line_content: line.to_string(),
                match_text: pattern.to_string(),
            });

            start = col_end;

            if results.len() >= 500 {
                return;
            }
        }
    }
}

fn is_markdown_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext.to_lowercase().as_str(), "md" | "markdown" | "txt")
    } else {
        false
    }
}
