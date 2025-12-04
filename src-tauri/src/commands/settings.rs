use crate::AppState;
use tauri::{AppHandle, State, Emitter};
use serde::{Deserialize, Serialize};
use crate::services::encryption::{encrypt_api_key, decrypt_api_key};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub api_key: String,
    pub theme: String,
    pub target_language: String,
    pub common_target_languages: Vec<String>,
}

// Helper to get value from DB
async fn get_val(pool: &sqlx::SqlitePool, key: &str) -> Option<String> {
    sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    // Encrypt API Key
    let encrypted_key = encrypt_api_key(&settings.api_key)?;

    // Save to DB
    let common_targets_json = serde_json::to_string(&settings.common_target_languages)
        .map_err(|e| e.to_string())?;

    let queries = [
        ("api_key", encrypted_key),
        ("theme", settings.theme.clone()),
        ("target_language", settings.target_language.clone()),
        ("common_target_languages", common_targets_json),
    ];

    for (key, value) in queries {
        sqlx::query("INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)")
            .bind(key)
            .bind(value)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Emit event to all windows
    app.emit("settings-changed", &settings).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let encrypted_key = get_val(&state.db, "api_key").await.unwrap_or_default();
    let theme = get_val(&state.db, "theme").await.unwrap_or_else(|| "dark".to_string());
    let target_language = get_val(&state.db, "target_language").await.unwrap_or_else(|| "zh-CN".to_string());
    let common_target_languages = get_val(&state.db, "common_target_languages").await
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_else(|| vec!["zh-CN".to_string(), "en".to_string(), "ja".to_string(), "ko".to_string()]);

    let api_key = if !encrypted_key.is_empty() {
        decrypt_api_key(&encrypted_key).unwrap_or_default()
    } else {
        String::new()
    };

    Ok(AppSettings {
        api_key,
        theme,
        target_language,
        common_target_languages,
    })
}
