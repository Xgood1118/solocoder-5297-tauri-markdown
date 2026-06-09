use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
