use tauri::{Manager, WindowEvent};

mod commands;
mod services;
mod models;
mod utils;
mod config;

// 导出宏供其他模块使用
pub use services::logger;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub main_pinned: std::sync::Arc<std::sync::Mutex<bool>>,
    pub main_loading: std::sync::Arc<std::sync::Mutex<bool>>,
    pub main_abort_handles: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u64, std::collections::HashMap<String, futures::future::AbortHandle>>>>,
    pub hotkey_double_copy_enabled: std::sync::Arc<std::sync::Mutex<bool>>,
    pub hotkey_alt_space_enabled: std::sync::Arc<std::sync::Mutex<bool>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    crate::app_info!("应用启动中...");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Focus existing instance instead of spawning another when launched again (e.g. from a shortcut)
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            crate::app_info!("开始初始化应用...");

            // macOS: 确保主窗口在激活时跟随当前桌面，避免快捷键唤起时切回旧的 Space。
            if let Some(window) = app.get_webview_window("main") {
                crate::utils::window::ensure_window_on_current_space(&window);
            }

            // 1. Initialize Database and Register Shortcut
            let handle = app.handle().clone();

            tauri::async_runtime::block_on(async move {
                crate::app_info!("正在初始化数据库...");
                let pool = match services::database::initialize_db(&handle).await {
                    Ok(pool) => {
                        crate::app_info!("数据库初始化成功");
                        pool
                    }
                    Err(e) => {
                        crate::app_error!("数据库初始化失败: {}", e);
                        return;
                    }
                };

                handle.manage(AppState {
                    db: pool.clone(),
                    main_pinned: std::sync::Arc::new(std::sync::Mutex::new(false)),
                    main_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
                    main_abort_handles: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
                    hotkey_double_copy_enabled: std::sync::Arc::new(std::sync::Mutex::new(true)),
                    hotkey_alt_space_enabled: std::sync::Arc::new(std::sync::Mutex::new(true)),
                });

                // Load hotkey config from DB
                let double_copy: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'hotkey_double_copy'")
                    .fetch_optional(&pool)
                    .await
                    .unwrap_or(None);
                let alt_space: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'hotkey_alt_space'")
                    .fetch_optional(&pool)
                    .await
                    .unwrap_or(None);

                if let Some(state) = handle.try_state::<AppState>() {
                    if let Some(val) = double_copy {
                        if let Ok(mut guard) = state.hotkey_double_copy_enabled.lock() {
                            *guard = val == "true";
                        }
                    }
                    if let Some(val) = alt_space {
                        if let Ok(mut guard) = state.hotkey_alt_space_enabled.lock() {
                            *guard = val == "true";
                        }
                    }
                }
            });

            // Start passive key listener (using platform-specific impl)
            crate::app_info!("正在启动热键监听器...");
            services::hotkey::start_listener(app.handle().clone());

            // 3. Initialize System Tray
            crate::app_info!("正在初始化系统托盘...");
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            let show = MenuItemBuilder::new("打开翻译窗口").id("show").build(app)?;
            let settings = MenuItemBuilder::new("打开设置").id("settings").build(app)?;
            let history = MenuItemBuilder::new("历史与日志").id("history").build(app)?;
            let quit = MenuItemBuilder::new("退出").id("quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &settings, &history, &quit]).build()?;

            // Load and decode icon
            let icon_bytes = include_bytes!("../icons/icon.png");
            let icon_img = image::load_from_memory(icon_bytes).expect("Failed to load icon");
            let width = icon_img.width();
            let height = icon_img.height();
            let rgba = icon_img.into_rgba8().into_vec();
            let icon = tauri::image::Image::new_owned(rgba, width, height);

            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "quit" => app.exit(0),
                        "show" => {
                            // 屏幕居中显示主窗口
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = commands::system::center_window_on_screen(&window);
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "settings" => {
                            if let Some(window) = app.get_webview_window("settings") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "history" => {
                            if let Some(window) = app.get_webview_window("history") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            // 托盘左键点击：屏幕居中显示主窗口
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                let _ = commands::system::center_window_on_screen(&window);
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            crate::app_info!("系统托盘初始化完成");

            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            crate::app_info!("应用初始化完成");
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                // 所有窗口关闭时只隐藏不退出
                if window.label() == "main" || window.label() == "settings" || window.label() == "history" {
                    window.hide().unwrap();
                    api.prevent_close();
                }
            }
            WindowEvent::Focused(false) => {
                if window.label() == "main" {
                    // 使用延迟检测，避免 Tauri drag-region bug 导致的短暂失焦
                    // See: https://github.com/tauri-apps/tauri/issues/10767
                    let app_handle = window.app_handle().clone();
                    let window_clone = window.clone();

                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                        // 如果窗口已重新获得焦点，说明是点击了窗口内部（如标题栏）
                        if window_clone.is_focused().unwrap_or(false) {
                            return;
                        }

                        let state: tauri::State<AppState> = app_handle.state();
                        let pinned = state.main_pinned.lock().map(|g| *g).unwrap_or(false);
                        let loading = state.main_loading.lock().map(|g| *g).unwrap_or(false);

                        if !pinned && !loading {
                            let _ = window_clone.hide();
                        }
                    });
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::translation::translate_text,
            commands::translation::translate_multi,
            commands::translation::translate_multi_stream_individual,
            commands::translation::cancel_translation,
            commands::translation::cancel_all_translations,
            commands::settings::save_settings,
            commands::settings::get_settings,
            commands::settings::get_provider_configs,
            commands::settings::save_provider_config,
            commands::settings::test_provider,
            commands::settings::get_hotkey_config,
            commands::settings::save_hotkey_config,
            commands::system::show_main_window,
            commands::system::show_main_window_centered,
            commands::system::show_settings_window,
            commands::system::show_history_window,
            commands::system::hide_window,
            commands::system::set_main_pinned,
            commands::system::get_main_pinned,
            commands::system::set_main_loading,
            commands::system::get_main_loading,
            commands::system::resize_main_window,
            commands::system::cache_stats,
            commands::system::clear_cache,
            commands::system::check_accessibility,
            commands::system::request_accessibility,
            commands::system::get_logs,
            commands::system::clear_logs,
            commands::system::get_translation_history,
            commands::system::delete_history_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
