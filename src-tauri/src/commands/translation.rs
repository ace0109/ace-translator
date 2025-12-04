use crate::services::zhipu;
use crate::AppState;
use tauri::{AppHandle, State};
use crate::services::encryption::decrypt_api_key;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
#[tauri::command]
pub async fn translate_text(
    _app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] targetLangs: Vec<String>,
    #[allow(non_snake_case)] requestId: Option<u64>,
) -> Result<Value, String> {
    let req_id = requestId.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    });

    println!(
        "[translate_text] start | req_id={:?} target_langs={:?} text_len={}",
        req_id,
        targetLangs,
        text.len()
    );

    if text.trim().is_empty() {
        return Err("Text is empty".to_string());
    }

    let encrypted_key: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'api_key'")
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let api_key = if let Some(k) = encrypted_key {
        decrypt_api_key(&k)?
    } else {
        return Err("API Key not set. Please configure it in settings.".to_string());
    };

    let mut cleaned = targetLangs.clone();
    cleaned.sort();
    cleaned.dedup();
    if cleaned.len() > 5 {
        cleaned.truncate(5);
    }
    if cleaned.is_empty() {
        cleaned.push("zh-CN".to_string());
    }

    // 在当前实现中，取消直接返回错误，调用方忽略结果即可
    if let Ok(mut loading) = state.floating_loading.lock() {
        *loading = true;
    }

    let result = zhipu::call_zhipu_api(&api_key, &text, "auto", &cleaned, Some(req_id)).await;

    if let Ok(mut loading) = state.floating_loading.lock() {
        *loading = false;
    }

    result
}
