use tauri::{AppHandle, Manager, WebviewWindow};
use crate::AppState;
use crate::services::logger::{LOGGER, LogEntry};
use serde::{Deserialize, Serialize};

/// 翻译历史条目
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HistoryEntry {
    pub id: i64,
    pub source_text: String,
    pub translated_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub created_at: String,
}

/// 将窗口水平居中，垂直方向距离屏幕顶部 5%
pub fn center_window_on_screen(window: &WebviewWindow) -> Result<(), String> {
    // 获取窗口当前大小
    let window_size = window.outer_size().map_err(|e| e.to_string())?;

    // 获取窗口所在的显示器
    if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
        let monitor_size = monitor.size();
        let monitor_position = monitor.position();

        // 水平居中
        let x = monitor_position.x + ((monitor_size.width as i32 - window_size.width as i32) / 2);
        // 垂直方向距离顶部 5%
        let y = monitor_position.y + (monitor_size.height as f64 * 0.05) as i32;

        window.set_position(tauri::PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn show_main_window_centered(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        center_window_on_screen(&window)?;
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
pub async fn show_history_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("history") {
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
pub async fn set_main_pinned(state: tauri::State<'_, AppState>, pinned: bool) -> Result<(), String> {
    if let Ok(mut guard) = state.main_pinned.lock() {
        *guard = pinned;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_main_pinned(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    if let Ok(guard) = state.main_pinned.lock() {
        return Ok(*guard);
    }
    Ok(false)
}

#[tauri::command]
pub async fn set_main_loading(state: tauri::State<'_, AppState>, loading: bool) -> Result<(), String> {
    if let Ok(mut guard) = state.main_loading.lock() {
        *guard = loading;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_main_loading(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    if let Ok(guard) = state.main_loading.lock() {
        return Ok(*guard);
    }
    Ok(false)
}

/// 动态调整主窗口高度
#[tauri::command]
pub async fn resize_main_window(app: AppHandle, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        // 使用 LogicalSize 获取当前大小（逻辑像素）
        let current_size = window.inner_size().map_err(|e| e.to_string())?.to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);

        // 获取屏幕高度来限制最大高度（70%）
        let max_height = if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
            let monitor_size = monitor.size().to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);
            monitor_size.height * 0.7
        } else {
            800.0 // 默认最大高度
        };

        // 限制高度在最小值和最大值之间
        let min_height = 200.0;
        let final_height = height.max(min_height).min(max_height);

        // 只有当高度变化超过 1 像素时才调整，避免抖动
        if (current_size.height - final_height).abs() > 1.0 {
             window.set_size(tauri::LogicalSize::new(current_size.width, final_height))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
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

/// 检查 macOS 辅助功能权限状态
#[tauri::command]
pub fn check_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    {
        crate::services::hotkey::check_accessibility_permission()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true // 非 macOS 平台不需要此权限
    }
}

/// 请求 macOS 辅助功能权限（会打开系统偏好设置）
#[tauri::command]
pub fn request_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    {
        crate::services::hotkey::prompt_accessibility_permission()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true // 非 macOS 平台不需要此权限
    }
}

/// 获取应用日志
#[tauri::command]
pub fn get_logs() -> Vec<LogEntry> {
    LOGGER.get_logs()
}

/// 清空日志
#[tauri::command]
pub fn clear_logs() {
    LOGGER.clear();
}

/// 获取翻译历史（分页，按时间倒序）
#[tauri::command]
pub async fn get_translation_history(
    state: tauri::State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<HistoryEntry>, String> {
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let entries: Vec<HistoryEntry> = sqlx::query_as(
        r#"
        SELECT id, source_text, translated_text, source_lang, target_lang, provider, model,
               COALESCE(created_at, '') as created_at
        FROM translation_history
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(entries)
}

/// 删除单条翻译历史
#[tauri::command]
pub async fn delete_history_entry(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    sqlx::query("DELETE FROM translation_history WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
