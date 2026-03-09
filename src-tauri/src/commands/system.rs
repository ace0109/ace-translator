use crate::services::logger::{LogEntry, LOGGER};
use crate::AppState;
use enigo::Mouse;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
#[cfg(target_os = "macos")]
use std::process::Command;

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

        window
            .set_position(tauri::PhysicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// 将窗口移动到鼠标所在的屏幕后再居中（找不到时回落到当前/主屏）
pub fn center_window_on_active_screen(window: &WebviewWindow) -> Result<(), String> {
    // Enigo 需要 Settings，创建默认配置
    let settings = enigo::Settings::default();
    let enigo = enigo::Enigo::new(&settings).map_err(|e| e.to_string())?;
    let (cursor_x, cursor_y) = enigo.location().map_err(|e| e.to_string())?;

    // 通过 AppHandle 获取所有显示器，兼容多平台 API
    let monitors = window
        .app_handle()
        .available_monitors()
        .map_err(|e| e.to_string())?;
    let target_monitor = monitors
        .into_iter()
        .find(|m| {
            let pos = m.position();
            let size = m.size();
            cursor_x >= pos.x
                && cursor_x < pos.x + size.width as i32
                && cursor_y >= pos.y
                && cursor_y < pos.y + size.height as i32
        })
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "无法获取显示器信息".to_string())?;

    let window_size = window.outer_size().map_err(|e| e.to_string())?;
    let monitor_size = target_monitor.size();
    let monitor_position = target_monitor.position();

    let x = monitor_position.x + ((monitor_size.width as i32 - window_size.width as i32) / 2);
    let y = monitor_position.y + (monitor_size.height as f64 * 0.05) as i32;

    window
        .set_position(tauri::PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

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

/// 打开日志窗口
#[tauri::command]
pub async fn show_logs_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("logs") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 打开关于窗口
#[tauri::command]
pub async fn show_about_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("about") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 打开权限引导窗口
#[tauri::command]
pub async fn show_permissions_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("permissions") {
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
pub async fn set_main_pinned(
    state: tauri::State<'_, AppState>,
    pinned: bool,
) -> Result<(), String> {
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
pub async fn set_main_loading(
    state: tauri::State<'_, AppState>,
    loading: bool,
) -> Result<(), String> {
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
        let current_size = window
            .inner_size()
            .map_err(|e| e.to_string())?
            .to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);

        // 获取屏幕高度来限制最大高度（70%）
        let max_height =
            if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
                let monitor_size = monitor
                    .size()
                    .to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);
                monitor_size.height * 0.8
            } else {
                800.0 // 默认最大高度
            };

        // 限制高度在最小值和最大值之间
        let min_height = 200.0;
        let final_height = height.max(min_height).min(max_height);

        // 只有当高度变化超过 1 像素时才调整，避免抖动
        if (current_size.height - final_height).abs() > 1.0 {
            window
                .set_size(tauri::LogicalSize::new(current_size.width, final_height))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 动态调整关于窗口高度
#[tauri::command]
pub async fn resize_about_window(app: AppHandle, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("about") {
        // 使用 LogicalSize 获取当前大小（逻辑像素）
        let current_size = window
            .inner_size()
            .map_err(|e| e.to_string())?
            .to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);

        // 获取屏幕高度来限制最大高度（80%）
        let max_height =
            if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
                let monitor_size = monitor
                    .size()
                    .to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);
                monitor_size.height * 0.8
            } else {
                800.0 // 默认最大高度
            };

        // 限制高度在最小值和最大值之间
        let min_height = 240.0;
        let final_height = height.max(min_height).min(max_height);

        // 只有当高度变化超过 1 像素时才调整，避免抖动
        if (current_size.height - final_height).abs() > 1.0 {
            window
                .set_size(tauri::LogicalSize::new(current_size.width, final_height))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn cache_stats(state: tauri::State<'_, AppState>) -> Result<(i64, i64), String> {
    let size: i64 = sqlx::query_scalar(
        "SELECT IFNULL(SUM(LENGTH(translated_text)), 0) FROM translation_history",
    )
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

/// 打开指定的隐私与安全面板
#[tauri::command]
pub fn open_privacy_panel(panel: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let uri = match panel.as_str() {
            "accessibility" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
            "screen_recording" => "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenRecording",
            "input_monitoring" => "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
            "files" => "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles",
            _ => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        };

        Command::new("open")
            .arg(uri)
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = panel; // 消除未使用变量警告
        Err("仅支持 macOS".to_string())
    }
}

/// 获取 .app bundle 路径（用于拖拽授权）
#[tauri::command]
pub fn get_app_bundle_path() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        // /Applications/App.app/Contents/MacOS/app -> ancestors nth(3) -> /Applications/App.app
        if let Some(bundle_path) = exe_path.ancestors().nth(3) {
            return Ok(bundle_path.to_string_lossy().to_string());
        }
        Err("无法定位应用路径".to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("仅支持 macOS".to_string())
    }
}

/// 在 Finder 中显示应用（备用拖拽方式）
#[tauri::command]
pub fn reveal_app_in_finder() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let bundle = get_app_bundle_path()?;
        Command::new("open")
            .args(["-R", &bundle])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("仅支持 macOS".to_string())
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

#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
    pub version: String,
    pub dev_mode: bool,
}

/// 获取应用信息（版本、开发者模式状态）
#[tauri::command]
pub fn get_app_info(state: tauri::State<'_, AppState>) -> AppInfo {
    let dev_mode = state.dev_mode.lock().map(|g| *g).unwrap_or(false);
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        dev_mode,
    }
}

/// 启用开发者模式（连续点击版本号 7 次后调用）
#[tauri::command]
pub async fn enable_dev_mode(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        if let Ok(mut guard) = state.dev_mode.lock() {
            if *guard {
                // already enabled
                return Ok(());
            }
            *guard = true;
        }
    }

    // persist
    sqlx::query("INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES ('dev_mode', 'true', CURRENT_TIMESTAMP)")
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    // 更新托盘菜单
    crate::utils::tray::refresh_tray_menu(&app);

    // 广播事件给前端
    let _ = app.emit("dev-mode-changed", true);

    Ok(())
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
        "#,
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
