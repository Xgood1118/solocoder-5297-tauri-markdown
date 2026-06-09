use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub path: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_dirty: bool,
    pub is_encrypted: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: Uuid,
    pub file_path: String,
    pub content_hash: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub line_content: String,
    pub match_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub provider: SyncProvider,
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub bucket: Option<String>,
    pub region: Option<String>,
    pub remote_path: String,
    pub auto_sync: bool,
    pub sync_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncProvider {
    Webdav,
    S3,
    Disabled,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            provider: SyncProvider::Disabled,
            endpoint: String::new(),
            username: String::new(),
            password: String::new(),
            bucket: None,
            region: None,
            remote_path: "/".to_string(),
            auto_sync: false,
            sync_interval_secs: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFile {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub modified_at: DateTime<Utc>,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginType,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    Command,
    Renderer,
    Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub bg_color: String,
    pub text_color: String,
    pub accent_color: String,
    pub font_family: String,
    pub font_size: u32,
}
