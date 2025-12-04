use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Manager, Emitter};
use tokio::time::sleep;
use crate::services::clipboard;
use mouse_position::mouse_position::Mouse;
use std::cmp;

// Platform-specific imports
#[cfg(target_os = "macos")]
use core_graphics::event::{CGEventTap, CGEventTapLocation, CGEventTapPlacement, CGEventTapOptions, CGEventFlags, CGKeyCode, CGEventType};
#[cfg(target_os = "macos")]
use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};

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
                let position = Mouse::get_mouse_position();
                let (mut pos_x, mut pos_y) = (300i32, 200i32);
                if let Mouse::Position { x, y } = position {
                    pos_x = x;
                    pos_y = y;
                }
                
                let window_width = 400i32;
                let window_height = 150i32;
                let target_x = cmp::max(0, pos_x - window_width / 2);
                let target_y = cmp::max(0, pos_y - (window_height / 2) - 10);
                
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: target_x,
                    y: target_y,
                }));
                
                println!("[hotkey] Showing floating window at ({}, {})", target_x, target_y);
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
                                if guard.ctrl_down || guard.meta_down { // Use tracked state
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
            }
        ) {
            Ok(tap) => {
                println!("[hotkey] Event tap created successfully");
                tap
            },
            Err(e) => {
                eprintln!("[hotkey] Failed to create event tap: {:?}", e);
                return;
            }
        };

        let loop_source = tap.mach_port.create_runloop_source(0).expect("Failed to create runloop source");
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
                    } else if key == Key::KeyC { // Rdev has Key::KeyC directly
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
