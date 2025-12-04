use tauri::{Manager, WindowEvent};

mod commands;
mod services;
mod models;
mod utils;
mod config;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub floating_pinned: std::sync::Arc<std::sync::Mutex<bool>>,
    pub floating_loading: std::sync::Arc<std::sync::Mutex<bool>>,
    pub floating_abort_handles: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u64, futures::future::AbortHandle>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // 1. Initialize Database and Register Shortcut
            let handle = app.handle().clone();
            
            tauri::async_runtime::block_on(async move {
                let pool = match services::database::initialize_db(&handle).await {
                    Ok(pool) => pool,
                    Err(e) => {
                        eprintln!("Failed to init DB: {}", e);
                        return;
                    }
                };

                handle.manage(AppState { 
                    db: pool, 
                    floating_pinned: std::sync::Arc::new(std::sync::Mutex::new(false)),
                    floating_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
                    floating_abort_handles: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
                });
            });

            // Start passive key listener (using platform-specific impl)
            services::hotkey::start_listener(app.handle().clone());

            // 3. Initialize System Tray
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            let show = MenuItemBuilder::new("Show Translator").id("show").build(app)?;
            let settings = MenuItemBuilder::new("Settings").id("settings").build(app)?;
            let quit = MenuItemBuilder::new("Quit").id("quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &settings, &quit]).build()?;

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
                            if let Some(window) = app.get_webview_window("main") {
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
                        _ => {}
                    }
                })
                .build(app)?;

            #[cfg(debug_assertions)]
            {
                // 仅打开悬浮窗 DevTools，避免重复弹窗
                if let Some(window) = app.get_webview_window("floating") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "main" {
                    window.hide().unwrap();
                    api.prevent_close();
                }
                if window.label() == "settings" {
                    window.hide().unwrap();
                    api.prevent_close();
                }
                if window.label() == "floating" {
                    window.hide().unwrap();
                    api.prevent_close();
                }
            }
            WindowEvent::Focused(false) => {
                if window.label() == "floating" {
                    // 如果未固定，则失焦隐藏；若已固定，则不隐藏
                    let app_handle = window.app_handle();
                    let state: tauri::State<AppState> = app_handle.state();
                    let pinned = state
                        .floating_pinned
                        .lock()
                        .map(|g| *g)
                        .unwrap_or(false);
                    let loading = state
                        .floating_loading
                        .lock()
                        .map(|g| *g)
                        .unwrap_or(false);
                    if !pinned && !loading {
                        let _ = window.hide();
                    }
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::translation::translate_text,
            commands::settings::save_settings,
            commands::settings::get_settings,
            commands::system::show_floating_window,
            commands::system::show_settings_window,
            commands::system::hide_window,
            commands::system::set_floating_pinned,
            commands::system::get_floating_pinned,
            commands::system::set_floating_loading,
            commands::system::get_floating_loading,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

