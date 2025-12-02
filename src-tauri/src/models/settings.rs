// src-tauri/src/models/settings.rs
#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}