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

    let target_join = cleaned.join(",");

    // 缓存检查：按 source_text + target_langs
    if let Ok(Some(cached)) = sqlx::query_scalar::<_, String>(
        "SELECT translated_text FROM translation_history WHERE source_text = ? AND target_lang = ?",
    )
    .bind(&text)
    .bind(&target_join)
    .fetch_optional(&state.db)
    .await
    {
        if let Ok(val) = serde_json::from_str::<Value>(&cached) {
            println!("[translate_text] cache hit | req_id={} targets={}", req_id, target_join);
            return Ok(val);
        }
    }

    if let Ok(mut loading) = state.floating_loading.lock() {
        *loading = true;
    }

    let result = zhipu::call_zhipu_api(&api_key, &text, "auto", &cleaned, Some(req_id)).await;

    if let Ok(mut loading) = state.floating_loading.lock() {
        *loading = false;
    }

    // 写入缓存
    if let Ok(ref val) = result {
        let _ = sqlx::query(
            "INSERT OR REPLACE INTO translation_history (source_text, translated_text, source_lang, target_lang) VALUES (?, ?, ?, ?)",
        )
        .bind(&text)
        .bind(val.to_string())
        .bind("auto")
        .bind(&target_join)
        .execute(&state.db)
        .await;
    }

    result
}
