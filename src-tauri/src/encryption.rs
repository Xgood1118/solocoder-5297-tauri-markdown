use std::collections::HashMap;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};
use tauri::State;
use tokio::fs;
use std::path::Path;

use crate::error::AppResult;
use crate::state::AppState;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], String> {
    let mut output = [0u8; KEY_LEN];

    let params = Params::new(65536, 3, 1, Some(KEY_LEN))
        .map_err(|e| format!("Failed to create params: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .map_err(|e| format!("Failed to derive key: {}", e))?;

    Ok(output)
}

#[tauri::command]
pub async fn encrypt_file(
    input_path: String,
    output_path: String,
    password: String,
) -> AppResult<bool> {
    let input = Path::new(&input_path);
    let content = fs::read_to_string(input).await?;

    let encrypted = encrypt_content(&content, &password)?;
    fs::write(output_path, encrypted).await?;

    Ok(true)
}

#[tauri::command]
pub async fn decrypt_file(
    input_path: String,
    password: String,
) -> AppResult<String> {
    let input = Path::new(&input_path);
    let encrypted_data = fs::read_to_string(input).await?;

    let decrypted = decrypt_content(&encrypted_data, &password)?;
    Ok(decrypted)
}

pub fn encrypt_content(content: &str, password: &str) -> AppResult<String> {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);

    let key_bytes = derive_key(password, &salt)
        .map_err(|e| crate::error::AppError::Encryption(e))?;

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, content.as_bytes())
        .map_err(|e| crate::error::AppError::Encryption(format!("Encryption failed: {}", e)))?;

    let mut result = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(result))
}

pub fn decrypt_content(encrypted: &str, password: &str) -> AppResult<String> {
    let data = BASE64
        .decode(encrypted)
        .map_err(|e| crate::error::AppError::Encryption(format!("Invalid base64: {}", e)))?;

    if data.len() < SALT_LEN + NONCE_LEN {
        return Err(crate::error::AppError::Encryption(
            "Invalid encrypted data".into(),
        ));
    }

    let salt = &data[..SALT_LEN];
    let nonce_bytes = &data[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &data[SALT_LEN + NONCE_LEN..];

    let key_bytes = derive_key(password, salt)
        .map_err(|e| crate::error::AppError::Encryption(e))?;

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| crate::error::AppError::Encryption(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| crate::error::AppError::Encryption(format!("Invalid UTF-8: {}", e)))
}

#[tauri::command]
pub async fn set_password(
    state: State<'_, AppState>,
    password: String,
) -> AppResult<bool> {
    let config_dir = state
        .get_config_dir()
        .await
        .ok_or_else(|| crate::error::AppError::Internal("Config dir not initialized".into()))?;

    let password_hash = hash_password(&password);
    let config_path = config_dir.join("security.json");

    let config = serde_json::json!({
        "password_hash": password_hash,
        "encryption_enabled": true
    });

    fs::write(&config_path, serde_json::to_string_pretty(&config)?).await?;
    Ok(true)
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
