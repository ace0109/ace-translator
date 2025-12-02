use tauri::{Manager, Emitter, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;
use mouse_position::mouse_position::Mouse; // Import Mouse
use std::{cmp, sync::{Arc, Mutex}, time::{Duration, Instant}, thread};
use tokio::time::sleep;
use rdev::{listen, Event, EventType, Key};

#[derive(Default)]
struct DoubleTapState {
    last: Option<Instant>,
    ignore_next: bool,
    ctrl_down: bool,
    meta_down: bool,
}

fn start_passive_hotkey_listener(app: tauri::AppHandle) {
    let state = Arc::new(Mutex::new(DoubleTapState::default()));
    let app_handle = app.clone();

    thread::spawn(move || {
        let state = state.clone();
        if let Err(e) = listen(move |event: Event| {
            let mut guard = state.lock().unwrap();
            match event.event_type {
                EventType::KeyPress(key) => {
                    if matches!(key, Key::ControlLeft | Key::ControlRight) {
                        guard.ctrl_down = true;
                        guard.ignore_next = false;
                    } else if matches!(key, Key::MetaLeft | Key::MetaRight) {
                        guard.meta_down = true;
                        guard.ignore_next = false;
                    } else if key == Key::KeyC {
                        if !(guard.ctrl_down || guard.meta_down) {
                            return;
                        }
                        let now = Instant::now();
                        let is_double = guard.last.map_or(false, |prev| now.duration_since(prev) <= Duration::from_millis(450));
                        if !is_double {
                            guard.last = Some(now);
                            return;
                        }
                        guard.last = None;
                        if guard.ignore_next {
                            guard.ignore_next = false;
                            return;
                        }
                        guard.ignore_next = true;

                        let app_clone = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            println!("[hotkey] double Ctrl/Cmd+C detected (passive)");
                            sleep(Duration::from_millis(40)).await;
                            match services::clipboard::read_clipboard(&app_clone) {
                                Ok(text) => {
                                    if text.trim().is_empty() {
                                        println!("[hotkey] skip: clipboard empty");
                                        return;
                                    }
                                    println!(
                                        "[hotkey] clipboard len={} preview=\"{}\"",
                                        text.len(),
                                        text.chars().take(80).collect::<String>()
                                    );
                                    if let Some(window) = app_clone.get_webview_window("floating") {
                                        let position = Mouse::get_mouse_position();
                                        let (mut pos_x, mut pos_y) = (300i32, 200i32);
                                        if let Mouse::Position { x, y } = position {
                                            pos_x = x;
                                            pos_y = y;
                                            println!("[hotkey] mouse position: x={}, y={}", x, y);
                                        }
                                        let window_width = 400i32;
                                        let window_height = 150i32;
                                        let target_x = cmp::max(0, pos_x - window_width / 2);
                                        let target_y = cmp::max(0, pos_y - (window_height / 2) - 10);
                                        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                                            x: target_x,
                                            y: target_y,
                                        }));
                                        println!("[hotkey] show floating at x={} y={}", target_x, target_y);
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                        sleep(Duration::from_millis(30)).await;
                                        match window.emit("floating-show", text.clone()) {
                                            Ok(_) => {
                                                println!("[hotkey] emit floating-show success, clearing clipboard");
                                                if let Err(e) = app_clone.clipboard().write_text("") {
                                                    eprintln!("[hotkey] clear clipboard failed: {}", e);
                                                }
                                            }
                                            Err(e) => eprintln!("[hotkey] emit floating-show failed: {}", e),
                                        }
                                    }
                                }
                                Err(e) => eprintln!("[hotkey] Failed to read clipboard: {}", e),
                            }
                        });
                    }
                }
                EventType::KeyRelease(key) => {
                    if matches!(key, Key::ControlLeft | Key::ControlRight) {
                        guard.ctrl_down = false;
                    } else if matches!(key, Key::MetaLeft | Key::MetaRight) {
                        guard.meta_down = false;
                    }
                    if matches!(key, Key::ControlLeft | Key::ControlRight | Key::MetaLeft | Key::MetaRight) {
                        guard.last = None;
                        guard.ignore_next = false;
                    }
                }
                _ => {}
            }
        }) {
            eprintln!("[hotkey] listener error: {:?}", e);
        }
    });
}

mod commands;
mod services;
mod models;
mod utils;

pub struct AppState {
    pub db: sqlx::SqlitePool,
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

                handle.manage(AppState { db: pool });
            });

            // Start passive key listener (Ctrl/Cmd + double C within 450ms)
            start_passive_hotkey_listener(app.handle().clone());

            // 3. Initialize System Tray
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            let show = MenuItemBuilder::new("Show Translator").id("show").build(app)?;
            let quit = MenuItemBuilder::new("Quit").id("quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

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
            }
            WindowEvent::Focused(false) => {
                if window.label() == "floating" {
                    let _ = window.hide();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::translation::translate_text,
            commands::settings::save_settings,
            commands::settings::get_settings,
            commands::system::show_floating_window,
            commands::system::hide_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

