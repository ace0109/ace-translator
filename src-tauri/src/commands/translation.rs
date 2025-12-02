use crate::services::zhipu;
use crate::AppState;
use tauri::{AppHandle, State};
use crate::services::encryption::decrypt_api_key;

#[tauri::command]
pub async fn translate_text(
    _app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    source_lang: String,
    target_lang: String,
) -> Result<String, String> {
    println!(
        "[translate_text] start | source_lang={} target_lang={} text_len={}",
        source_lang,
        target_lang,
        text.len()
    );

    // 1) 缓存查询
    let cached_translation: Option<String> = sqlx::query_scalar(
        "SELECT translated_text FROM translation_history WHERE source_text = ? AND source_lang = ? AND target_lang = ?"
    )
    .bind(&text)
    .bind(&source_lang)
    .bind(&target_lang)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(translation) = cached_translation {
        println!("[translate_text] cache hit, returning cached result");
        return Ok(translation);
    }

    // 2) 取 API Key
    let encrypted_key: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'api_key'")
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let api_key = if let Some(k) = encrypted_key {
        decrypt_api_key(&k)?
    } else {
        return Err("API Key not set. Please configure it in settings.".to_string());
    };

    // 3) 调用 API
    let translated = zhipu::call_zhipu_api(&api_key, &text, &source_lang, &target_lang).await?;
    println!(
        "[translate_text] api done | translated_len={} preview=\"{}\"",
        translated.len(),
        translated.chars().take(60).collect::<String>()
    );

    // 4) 写入历史
    let _ = sqlx::query(
        "INSERT INTO translation_history (source_text, translated_text, source_lang, target_lang) VALUES (?, ?, ?, ?)"
    )
    .bind(&text)
    .bind(&translated)
    .bind(&source_lang)
    .bind(&target_lang)
    .execute(&state.db)
    .await;

    Ok(translated)
}
