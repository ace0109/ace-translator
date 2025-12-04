use crate::services::zhipu;
use crate::AppState;
use tauri::{AppHandle, State};
use crate::services::encryption::decrypt_api_key;
use serde_json::Value;

#[tauri::command]
pub async fn translate_text(
    _app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    // 目标语言列表（前端已去重&限制长度，后端再兜底限制）
    #[allow(non_snake_case)]
    targetLangs: Vec<String>,
) -> Result<Value, String> {
    println!(
        "[translate_text] start | target_langs={:?} text_len={}",
        targetLangs,
        text.len()
    );

    if text.trim().is_empty() {
        return Err("Text is empty".to_string());
    }

    // 拉取 API Key
    let encrypted_key: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'api_key'")
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let api_key = if let Some(k) = encrypted_key {
        decrypt_api_key(&k)?
    } else {
        return Err("API Key not set. Please configure it in settings.".to_string());
    };

    // 最多 5 个目标语言，去重
    let mut cleaned = targetLangs.clone();
    cleaned.sort();
    cleaned.dedup();
    if cleaned.len() > 5 {
        cleaned.truncate(5);
    }
    if cleaned.is_empty() {
        cleaned.push("zh-CN".to_string());
    }

    let result = zhipu::call_zhipu_api(&api_key, &text, "auto", &cleaned).await?;

    Ok(result)
}
