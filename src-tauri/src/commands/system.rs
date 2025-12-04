use tauri::{AppHandle, Manager};
use crate::AppState;

#[tauri::command]
pub async fn show_floating_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn show_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_window(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_floating_pinned(state: tauri::State<'_, AppState>, pinned: bool) -> Result<(), String> {
    if let Ok(mut guard) = state.floating_pinned.lock() {
        *guard = pinned;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_floating_pinned(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    if let Ok(guard) = state.floating_pinned.lock() {
        return Ok(*guard);
    }
    Ok(false)
}

#[tauri::command]
pub async fn set_floating_loading(state: tauri::State<'_, AppState>, loading: bool) -> Result<(), String> {
    if let Ok(mut guard) = state.floating_loading.lock() {
        *guard = loading;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_floating_loading(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    if let Ok(guard) = state.floating_loading.lock() {
        return Ok(*guard);
    }
    Ok(false)
}

#[tauri::command]
pub async fn cache_stats(state: tauri::State<'_, AppState>) -> Result<(i64, i64), String> {
    let size: i64 = sqlx::query_scalar("SELECT IFNULL(SUM(LENGTH(translated_text)), 0) FROM translation_history")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM translation_history")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    Ok((count, size))
}

#[tauri::command]
pub async fn clear_cache(state: tauri::State<'_, AppState>) -> Result<(), String> {
    sqlx::query("DELETE FROM translation_history")
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
