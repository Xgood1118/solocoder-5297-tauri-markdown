use std::path::Path;
use grep_regex::RegexMatcher;
use grep_searcher::sinks::UTF8;
use grep_searcher::{Searcher, SearcherBuilder};
use ignore::WalkBuilder;

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

    let walker = WalkBuilder::new(dir_path)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .follow_links(false)
        .max_depth(None)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_file() {
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
    let matcher = RegexMatcher::new(&pattern)
        .map_err(|e| crate::error::AppError::Search(format!("Invalid pattern: {}", e)))?;

    let mut searcher = SearcherBuilder::new()
        .encoding(None)
        .line_number(true)
        .word(false)
        .build();

    let walker = WalkBuilder::new(dir_path)
        .hidden(false)
        .git_ignore(true)
        .types(grep_cli::default_types().to_owned())
        .build();

    let results: std::sync::Mutex<Vec<SearchResult>> = std::sync::Mutex::new(Vec::new());

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext != "md" && ext != "markdown" && ext != "txt" {
                    continue;
                }
            } else {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            let results_ref = &results;

            let _ = searcher.search_path(
                &matcher,
                path,
                UTF8(|lnum, line| {
                    let line_str = line.trim_end_matches('\n').trim_end_matches('\r');
                    let pattern_chars: Vec<char> = pattern.chars().collect();
                    let line_chars: Vec<char> = line_str.chars().collect();

                    let mut start = 0;
                    while let Some(pos) = find_substring(&line_chars[start..], &pattern_chars) {
                        let col_start = start + pos;
                        let col_end = col_start + pattern_chars.len();

                        let mut result = results_ref.lock().unwrap();
                        result.push(SearchResult {
                            file_path: path_str.clone(),
                            line_number: lnum as usize,
                            column_start: col_start + 1,
                            column_end: col_end + 1,
                            line_content: line_str.to_string(),
                            match_text: pattern.clone(),
                        });
                        drop(result);

                        start = col_end;
                    }

                    Ok(true)
                }),
            );
        }
    }

    let results = results.into_inner().unwrap();
    Ok(results)
}

fn find_substring(haystack: &[char], needle: &[char]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    for i in 0..=haystack.len() - needle.len() {
        if haystack[i..i + needle.len()] == *needle {
            return Some(i);
        }
    }
    None
}
