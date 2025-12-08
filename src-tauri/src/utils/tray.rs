use tauri::AppHandle;
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder};
use tauri::Manager;

/// 构建托盘菜单，根据 dev_mode 决定是否显示日志入口。
pub fn build_tray_menu(app: &AppHandle, dev_mode: bool) -> tauri::Result<Menu<tauri::Wry>> {
    let show_label = if cfg!(target_os = "macos") {
        "打开翻译窗口 (Option+Space)"
    } else {
        "打开翻译窗口 (Alt+Space)"
    };

    let show = MenuItemBuilder::new(show_label).id("show").build(app)?;
    let settings = MenuItemBuilder::new("打开设置").id("settings").build(app)?;
    let history = MenuItemBuilder::new("历史").id("history").build(app)?;
    let about = MenuItemBuilder::new("关于").id("about").build(app)?;
    let quit = MenuItemBuilder::new("退出").id("quit").build(app)?;

    let mut builder = MenuBuilder::new(app).items(&[&show, &settings, &history, &about]);

    if dev_mode {
        let logs = MenuItemBuilder::new("日志").id("logs").build(app)?;
        builder = builder.items(&[&logs]);
    }

    builder.items(&[&quit]).build()
}

/// 根据当前状态刷新托盘菜单。
pub fn refresh_tray_menu(app: &AppHandle) {
    if let Some(state) = app.try_state::<crate::AppState>() {
        let dev_mode = state.dev_mode.lock().map(|g| *g).unwrap_or(false);
        if let Ok(menu) = build_tray_menu(app, dev_mode) {
            if let Ok(guard) = state.tray_icon.lock() {
                if let Some(tray) = guard.as_ref() {
                    let _ = tray.set_menu(Some(menu));
                }
            }
        }
    }
}
