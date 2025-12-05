use std::cmp;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use mouse_position::mouse_position::Mouse;
use tauri::{Emitter, Manager};
use tokio::time::sleep;

use crate::{services::clipboard, AppState};

// Platform-specific imports
#[cfg(target_os = "macos")]
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
#[cfg(target_os = "macos")]
use core_graphics::event::{
    CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CGKeyCode,
};

#[cfg(any(target_os = "windows", target_os = "linux"))] // rdev supports Windows and Linux
use rdev::{listen, Event, EventType, Key};

#[derive(Default)]
struct DoubleTapState {
    last_c_press: Option<Instant>,
    ctrl_down: bool, // For Windows/Linux Ctrl, macOS Ctrl
    meta_down: bool, // For macOS Command, Windows/Linux Meta
}

// Key Codes for macOS
#[cfg(target_os = "macos")]
const KEY_C: CGKeyCode = 0x08;

// Platform-agnostic handle_double_copy
async fn handle_double_copy(app: tauri::AppHandle) {
    println!("[hotkey] Double Copy Detected!");
    sleep(Duration::from_millis(100)).await; // Give system time to update clipboard

    match clipboard::read_clipboard(&app) {
        Ok(text) => {
            if text.trim().is_empty() {
                println!("[hotkey] Skipping: clipboard empty or whitespace only.");
                return;
            }

            if let Some(window) = app.get_webview_window("floating") {
                // 若已固定，则不改坐标；未固定时按鼠标居中定位
                let pinned = {
                    let state: tauri::State<AppState> = app.state();
                    state.floating_pinned.lock().map(|g| *g).unwrap_or(false)
                };

                let loading = {
                    let state: tauri::State<AppState> = app.state();
                    state.floating_loading.lock().map(|g| *g).unwrap_or(false)
                };

                if !pinned && !loading {
                    // 使用窗口真实外部尺寸（考虑缩放/装饰）进行边界收缩，避免 DPI 与多屏溢出
                    let outer_size = window
                        .outer_size()
                        .unwrap_or(tauri::PhysicalSize::new(400, 500));

                    let position = Mouse::get_mouse_position();
                    let (mut pos_x, mut pos_y) = (300i32, 200i32);
                    if let Mouse::Position { x, y } = position {
                        pos_x = x;
                        pos_y = y;
                    }

                    #[cfg(target_os = "macos")]
                    {
                        // macOS logic: mouse_position returns Logical points.
                        // We must convert window size and monitor bounds to Logical points to match.
                        let scale_factor = window.scale_factor().unwrap_or(1.0);
                        let win_w = outer_size.width as f64 / scale_factor;
                        let win_h = outer_size.height as f64 / scale_factor;

                        let m_x = pos_x as f64;
                        let m_y = pos_y as f64;

                        // Target center
                        let mut target_x = m_x - win_w / 2.0;
                        let mut target_y = m_y - (win_h / 2.0) - 10.0;

                        if let Ok(monitors) = app.available_monitors() {
                            // Find monitor containing mouse (using Logical bounds)
                            let monitor = monitors
                                .iter()
                                .find(|m| {
                                    let scale = m.scale_factor();
                                    let pos = m.position(); // Physical
                                    let size = m.size(); // Physical

                                    let min_x = pos.x as f64 / scale;
                                    let min_y = pos.y as f64 / scale;
                                    let max_x = min_x + (size.width as f64 / scale);
                                    let max_y = min_y + (size.height as f64 / scale);

                                    m_x >= min_x && m_x <= max_x && m_y >= min_y && m_y <= max_y
                                })
                                .or(monitors.first());

                            if let Some(m) = monitor {
                                let scale = m.scale_factor();
                                let pos = m.position();
                                let size = m.size();

                                let min_x = pos.x as f64 / scale;
                                let min_y = pos.y as f64 / scale;
                                // Constrain target so window stays within monitor
                                let max_x = min_x + (size.width as f64 / scale) - win_w;
                                let max_y = min_y + (size.height as f64 / scale) - win_h;

                                target_x = target_x.clamp(min_x, max_x.max(min_x));
                                target_y = target_y.clamp(min_y, max_y.max(min_y));
                            }
                        }

                        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
                            x: target_x,
                            y: target_y,
                        }));
                        println!(
                            "[hotkey] Showing floating window at Logical ({}, {})",
                            target_x, target_y
                        );
                    }

                    #[cfg(not(target_os = "macos"))]
                    {
                        // Windows/Linux logic: mouse_position returns Physical pixels (usually).
                        let window_width = outer_size.width as i32;
                        let window_height = outer_size.height as i32;

                        // 目标：大致居中到鼠标附近
                        let mut target_x = pos_x - window_width / 2;
                        let mut target_y = pos_y - (window_height / 2) - 10;

                        if let Ok(monitors) = app.available_monitors() {
                            // 优先选包含鼠标的显示器进行边界约束
                            if let Some(mon) = monitors.iter().find(|m| {
                                let pos = m.position();
                                let size = m.size();
                                let x0 = pos.x;
                                let y0 = pos.y;
                                let x1 = x0 + size.width as i32;
                                let y1 = y0 + size.height as i32;
                                pos_x >= x0 && pos_x <= x1 && pos_y >= y0 && pos_y <= y1
                            }) {
                                let pos = mon.position();
                                let size = mon.size();
                                let min_x = pos.x;
                                let min_y = pos.y;
                                let max_x = pos.x + size.width as i32 - window_width;
                                let max_y = pos.y + size.height as i32 - window_height;
                                target_x = target_x.clamp(min_x, cmp::max(min_x, max_x));
                                target_y = target_y.clamp(min_y, cmp::max(min_y, max_y));
                            } else if let Some(primary) = monitors.first() {
                                // 找不到匹配显示器时，至少落在第一个显示器范围内
                                let pos = primary.position();
                                let size = primary.size();
                                let min_x = pos.x;
                                let min_y = pos.y;
                                let max_x = pos.x + size.width as i32 - window_width;
                                let max_y = pos.y + size.height as i32 - window_height;
                                target_x = target_x.clamp(min_x, cmp::max(min_x, max_x));
                                target_y = target_y.clamp(min_y, cmp::max(min_y, max_y));
                            }
                        }

                        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                            x: target_x,
                            y: target_y,
                        }));

                        println!(
                            "[hotkey] Showing floating window at Physical ({}, {})",
                            target_x, target_y
                        );
                    }
                } else {
                    println!("[hotkey] Floating pinned, keep position");
                }

                let _ = window.show();
                let _ = window.set_focus();

                sleep(Duration::from_millis(50)).await;
                let _ = window.emit("floating-show", text);
            } else {
                eprintln!("[hotkey] Floating window not found!");
            }
        }
        Err(e) => eprintln!("[hotkey] Failed to read clipboard: {}", e),
    }
}

#[cfg(target_os = "macos")]
pub fn start_listener(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let state = Arc::new(Mutex::new(DoubleTapState::default()));

        let tap = match CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::KeyDown, CGEventType::FlagsChanged],
            {
                let app_handle = app.clone();
                let state = state.clone();
                move |_proxy, type_, event| {
                    let mut guard = state.lock().unwrap(); // Lock for the entire event processing.

                    match type_ {
                        CGEventType::KeyDown => {
                            let key_code = event.get_integer_value_field(9) as CGKeyCode; // kCGKeyboardEventKeycode = 9

                            // Use tracked modifier state
                            // println!("[hotkey] KeyDown: code={}, ctrl_state={}, meta_state={}", key_code, guard.ctrl_down, guard.meta_down);

                            if key_code == KEY_C {
                                if guard.ctrl_down || guard.meta_down {
                                    // Use tracked state
                                    let now = Instant::now();

                                    let is_double = guard.last_c_press.map_or(false, |prev| {
                                        let diff = now.duration_since(prev);
                                        // println!("[hotkey] Time since last press: {:?}", diff);
                                        diff <= Duration::from_millis(450)
                                    });

                                    if !is_double {
                                        // println!("[hotkey] First Cmd+C detected");
                                        guard.last_c_press = Some(now);
                                    } else {
                                        // println!("[hotkey] Second Cmd+C detected! Triggering...");
                                        guard.last_c_press = None;

                                        let app_clone = app_handle.clone();
                                        tauri::async_runtime::spawn(async move {
                                            handle_double_copy(app_clone).await;
                                        });
                                    }
                                }
                            }
                        }
                        CGEventType::FlagsChanged => {
                            let flags = event.get_flags();
                            guard.ctrl_down = flags.contains(CGEventFlags::CGEventFlagControl);
                            guard.meta_down = flags.contains(CGEventFlags::CGEventFlagCommand);
                            // println!("[hotkey] FlagsChanged: ctrl={}, meta={}", guard.ctrl_down, guard.meta_down);
                            // Also reset double tap state if modifiers are released
                            if !guard.ctrl_down && !guard.meta_down {
                                guard.last_c_press = None;
                                // println!("[hotkey] Modifiers released, reset double tap state.");
                            }
                        }
                        _ => {}
                    }
                    Some(event.to_owned())
                }
            },
        ) {
            Ok(tap) => {
                println!("[hotkey] Event tap created successfully");
                tap
            }
            Err(e) => {
                eprintln!("[hotkey] Failed to create event tap: {:?}", e);
                return;
            }
        };

        let loop_source = tap
            .mach_port
            .create_runloop_source(0)
            .expect("Failed to create runloop source");
        let current_loop = CFRunLoop::get_current();
        current_loop.add_source(&loop_source, unsafe { kCFRunLoopCommonModes });

        tap.enable();
        CFRunLoop::run_current();
    });
}

#[cfg(target_os = "windows")]
pub fn start_listener(app: tauri::AppHandle) {
    let state = Arc::new(Mutex::new(DoubleTapState::default()));
    let app_handle = app.clone();

    std::thread::spawn(move || {
        let state = state.clone();
        if let Err(e) = listen(move |event: Event| {
            let mut guard = state.lock().unwrap();
            match event.event_type {
                EventType::KeyPress(key) => {
                    if matches!(key, Key::ControlLeft | Key::ControlRight) {
                        guard.ctrl_down = true;
                    } else if matches!(key, Key::MetaLeft | Key::MetaRight) {
                        guard.meta_down = true;
                    } else if key == Key::KeyC {
                        // Rdev has Key::KeyC directly
                        if !(guard.ctrl_down || guard.meta_down) {
                            // If neither Ctrl nor Meta is down, it's not a Ctrl/Cmd+C
                            return;
                        }
                        let now = Instant::now();
                        let is_double = guard.last_c_press.map_or(false, |prev| {
                            let diff = now.duration_since(prev);
                            println!("[hotkey] Time since last press: {:?}", diff);
                            diff <= Duration::from_millis(450)
                        });

                        if !is_double {
                            println!("[hotkey] First Ctrl/Cmd+C detected");
                            guard.last_c_press = Some(now);
                        } else {
                            println!("[hotkey] Second Ctrl/Cmd+C detected! Triggering...");
                            guard.last_c_press = None;

                            let app_clone = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                handle_double_copy(app_clone).await;
                            });
                        }
                    }
                }
                EventType::KeyRelease(key) => {
                    if matches!(key, Key::ControlLeft | Key::ControlRight) {
                        guard.ctrl_down = false;
                        guard.last_c_press = None; // Reset double tap state if modifier released
                    } else if matches!(key, Key::MetaLeft | Key::MetaRight) {
                        guard.meta_down = false;
                        guard.last_c_press = None; // Reset double tap state if modifier released
                    }
                }
                _ => {}
            }
        }) {
            eprintln!("[hotkey] Listener error on Windows: {:?}", e);
        }
    });
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn start_listener(_app: tauri::AppHandle) {
    println!("Hotkey listener not implemented for this OS yet in this custom module. (Linux/Other)");
}
