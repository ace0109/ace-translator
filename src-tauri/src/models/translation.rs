// src-tauri/src/models/translation.rs
#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Translation {
    pub id: Option<i64>,
    pub source_text: String,
    pub translated_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub created_at: String,
}
