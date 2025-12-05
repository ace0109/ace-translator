use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use mouse_position::mouse_position::Mouse;
use tauri::{Emitter, Manager};
use tokio::time::sleep;

use crate::{services::clipboard, AppState, app_info, app_error, app_debug};

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

/// 检查 macOS 辅助功能权限
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    let result = unsafe { AXIsProcessTrusted() };
    app_info!("检查辅助功能权限: {}", if result { "已授权" } else { "未授权" });
    result
}

/// 请求用户授予辅助功能权限（打开系统偏好设置）
#[cfg(target_os = "macos")]
pub fn prompt_accessibility_permission() -> bool {
    use std::ffi::c_void;
    use std::ptr;

    app_info!("正在请求辅助功能权限...");

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFDictionaryCreate(
            allocator: *const c_void,
            keys: *const *const c_void,
            values: *const *const c_void,
            num_values: isize,
            key_callbacks: *const c_void,
            value_callbacks: *const c_void,
        ) -> *const c_void;
        fn CFRelease(cf: *const c_void);
        static kCFTypeDictionaryKeyCallBacks: c_void;
        static kCFTypeDictionaryValueCallBacks: c_void;
        static kCFBooleanTrue: *const c_void;
    }

    // kAXTrustedCheckOptionPrompt key
    const K_AX_TRUSTED_CHECK_OPTION_PROMPT: &[u8] = b"AXTrustedCheckOptionPrompt\0";

    unsafe {
        // Create CFString for the key
        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            fn CFStringCreateWithCString(
                allocator: *const c_void,
                c_str: *const i8,
                encoding: u32,
            ) -> *const c_void;
        }

        let key = CFStringCreateWithCString(
            ptr::null(),
            K_AX_TRUSTED_CHECK_OPTION_PROMPT.as_ptr() as *const i8,
            0x08000100, // kCFStringEncodingUTF8
        );

        let keys = [key];
        let values = [kCFBooleanTrue];

        let options = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        );

        let result = AXIsProcessTrustedWithOptions(options);

        CFRelease(options);
        CFRelease(key);

        app_info!("辅助功能权限请求结果: {}", if result { "已授权" } else { "等待用户授权" });
        result
    }
}

// Platform-agnostic handle_double_copy
async fn handle_double_copy(app: tauri::AppHandle) {
    app_info!("检测到双击复制！开始处理悬浮窗显示...");
    sleep(Duration::from_millis(100)).await; // Give system time to update clipboard

    app_debug!("正在读取剪贴板内容...");
    match clipboard::read_clipboard(&app) {
        Ok(text) => {
            app_info!("剪贴板内容读取成功，长度: {} 字符", text.len());
            if text.trim().is_empty() {
                app_info!("跳过：剪贴板内容为空或仅包含空白字符");
                return;
            }

            app_debug!("正在获取悬浮窗口...");
            if let Some(window) = app.get_webview_window("floating") {
                app_info!("成功获取到悬浮窗口");

                // 若已固定，则不改坐标；未固定时按鼠标居中定位
                let pinned = {
                    let state: tauri::State<AppState> = app.state();
                    state.floating_pinned.lock().map(|g| *g).unwrap_or(false)
                };

                let loading = {
                    let state: tauri::State<AppState> = app.state();
                    state.floating_loading.lock().map(|g| *g).unwrap_or(false)
                };

                app_debug!("悬浮窗状态 - 固定: {}, 加载中: {}", pinned, loading);

                if !pinned && !loading {
                    // 使用窗口真实外部尺寸（考虑缩放/装饰）进行边界收缩，避免 DPI 与多屏溢出
                    let outer_size = window
                        .outer_size()
                        .unwrap_or(tauri::PhysicalSize::new(400, 500));

                    app_debug!("窗口外部尺寸: {}x{}", outer_size.width, outer_size.height);

                    let position = Mouse::get_mouse_position();
                    let (mut pos_x, mut pos_y) = (300i32, 200i32);
                    if let Mouse::Position { x, y } = position {
                        pos_x = x;
                        pos_y = y;
                    }
                    app_debug!("鼠标位置: ({}, {})", pos_x, pos_y);

                    #[cfg(target_os = "macos")]
                    {
                        // macOS logic: mouse_position returns Logical points.
                        // We must convert window size and monitor bounds to Logical points to match.
                        let scale_factor = window.scale_factor().unwrap_or(1.0);
                        app_debug!("缩放因子: {}", scale_factor);

                        let win_w = outer_size.width as f64 / scale_factor;
                        let win_h = outer_size.height as f64 / scale_factor;

                        let m_x = pos_x as f64;
                        let m_y = pos_y as f64;

                        // Target center
                        let mut target_x = m_x - win_w / 2.0;
                        let mut target_y = m_y - (win_h / 2.0) - 10.0;

                        if let Ok(monitors) = app.available_monitors() {
                            app_debug!("可用显示器数量: {}", monitors.len());
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
                        app_info!("设置悬浮窗位置（逻辑坐标）: ({}, {})", target_x, target_y);
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
                                target_x = target_x.clamp(min_x, std::cmp::max(min_x, max_x));
                                target_y = target_y.clamp(min_y, std::cmp::max(min_y, max_y));
                            } else if let Some(primary) = monitors.first() {
                                // 找不到匹配显示器时，至少落在第一个显示器范围内
                                let pos = primary.position();
                                let size = primary.size();
                                let min_x = pos.x;
                                let min_y = pos.y;
                                let max_x = pos.x + size.width as i32 - window_width;
                                let max_y = pos.y + size.height as i32 - window_height;
                                target_x = target_x.clamp(min_x, std::cmp::max(min_x, max_x));
                                target_y = target_y.clamp(min_y, std::cmp::max(min_y, max_y));
                            }
                        }

                        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                            x: target_x,
                            y: target_y,
                        }));

                        app_info!("设置悬浮窗位置（物理坐标）: ({}, {})", target_x, target_y);
                    }
                } else {
                    app_info!("悬浮窗已固定或加载中，保持当前位置");
                }

                app_debug!("正在显示悬浮窗...");
                match window.show() {
                    Ok(_) => app_info!("悬浮窗显示成功"),
                    Err(e) => app_error!("悬浮窗显示失败: {}", e),
                }

                match window.set_focus() {
                    Ok(_) => app_debug!("悬浮窗获得焦点"),
                    Err(e) => app_error!("悬浮窗获取焦点失败: {}", e),
                }

                sleep(Duration::from_millis(50)).await;

                app_debug!("正在发送 floating-show 事件...");
                match window.emit("floating-show", text.clone()) {
                    Ok(_) => app_info!("floating-show 事件发送成功，文本长度: {}", text.len()),
                    Err(e) => app_error!("floating-show 事件发送失败: {}", e),
                }
            } else {
                app_error!("无法获取悬浮窗口！窗口可能未创建或已销毁");
            }
        }
        Err(e) => app_error!("读取剪贴板失败: {}", e),
    }
}

#[cfg(target_os = "macos")]
pub fn start_listener(app: tauri::AppHandle) {
    app_info!("正在启动 macOS 热键监听器...");

    std::thread::spawn(move || {
        app_info!("热键监听线程已启动");
        let state = Arc::new(Mutex::new(DoubleTapState::default()));
        let event_count = Arc::new(Mutex::new(0u64));

        app_debug!("正在创建 CGEventTap...");
        let tap = match CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::KeyDown, CGEventType::FlagsChanged],
            {
                let app_handle = app.clone();
                let state = state.clone();
                let event_count = event_count.clone();
                move |_proxy, type_, event| {
                    // 记录事件计数
                    if let Ok(mut count) = event_count.lock() {
                        *count += 1;
                        // 每收到第一个事件时记录，证明回调在工作
                        if *count == 1 {
                            crate::services::logger::LOGGER.info("收到第一个键盘事件，回调函数正常工作");
                        }
                        // 每 100 个事件记录一次
                        if *count % 100 == 0 {
                            crate::services::logger::LOGGER.info(&format!("已处理 {} 个键盘事件", *count));
                        }
                    }

                    let mut guard = state.lock().unwrap();

                    match type_ {
                        CGEventType::KeyDown => {
                            let key_code = event.get_integer_value_field(9) as CGKeyCode;

                            // 记录所有按键（仅用于调试）
                            crate::services::logger::LOGGER.debug(&format!(
                                "KeyDown: keycode={}, meta={}, ctrl={}",
                                key_code, guard.meta_down, guard.ctrl_down
                            ));

                            if key_code == KEY_C {
                                crate::services::logger::LOGGER.info(&format!(
                                    "检测到 C 键按下，meta={}, ctrl={}",
                                    guard.meta_down, guard.ctrl_down
                                ));

                                if guard.ctrl_down || guard.meta_down {
                                    let now = Instant::now();

                                    let is_double = guard.last_c_press.map_or(false, |prev| {
                                        let diff = now.duration_since(prev);
                                        crate::services::logger::LOGGER.debug(&format!(
                                            "距离上次 Cmd+C 时间: {:?}ms",
                                            diff.as_millis()
                                        ));
                                        diff <= Duration::from_millis(450)
                                    });

                                    if !is_double {
                                        crate::services::logger::LOGGER.info("检测到第一次 Cmd/Ctrl+C");
                                        guard.last_c_press = Some(now);
                                    } else {
                                        crate::services::logger::LOGGER.info("检测到双击 Cmd/Ctrl+C！触发悬浮翻译...");
                                        guard.last_c_press = None;

                                        let app_clone = app_handle.clone();
                                        tauri::async_runtime::spawn(async move {
                                            handle_double_copy(app_clone).await;
                                        });
                                    }
                                } else {
                                    crate::services::logger::LOGGER.debug("C 键按下但没有修饰键，忽略");
                                }
                            }
                        }
                        CGEventType::FlagsChanged => {
                            let flags = event.get_flags();
                            let old_meta = guard.meta_down;
                            let old_ctrl = guard.ctrl_down;

                            guard.ctrl_down = flags.contains(CGEventFlags::CGEventFlagControl);
                            guard.meta_down = flags.contains(CGEventFlags::CGEventFlagCommand);

                            // 只在状态变化时记录
                            if old_meta != guard.meta_down || old_ctrl != guard.ctrl_down {
                                crate::services::logger::LOGGER.debug(&format!(
                                    "修饰键状态变化: meta={}, ctrl={}",
                                    guard.meta_down, guard.ctrl_down
                                ));
                            }

                            if !guard.ctrl_down && !guard.meta_down {
                                guard.last_c_press = None;
                            }
                        }
                        _ => {}
                    }
                    Some(event.to_owned())
                }
            },
        ) {
            Ok(tap) => {
                app_info!("CGEventTap 创建成功！热键监听已就绪");
                tap
            }
            Err(e) => {
                // EventTap 创建失败，说明没有辅助功能权限
                app_error!("CGEventTap 创建失败: {:?}", e);
                app_error!("这通常表示应用没有辅助功能权限");

                // 请求权限（弹出系统设置）
                app_info!("正在请求辅助功能权限...");
                let _ = prompt_accessibility_permission();

                // 发送事件通知前端
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("accessibility-permission-needed", ());
                }

                // 启动后台轮询，等待权限授予后重试
                app_info!("启动权限检测轮询（每3秒检查一次）...");
                let app_clone = app.clone();
                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(Duration::from_secs(3));

                        // 尝试创建 EventTap 来检测权限是否已授予
                        let test_tap = CGEventTap::new(
                            CGEventTapLocation::Session,
                            CGEventTapPlacement::HeadInsertEventTap,
                            CGEventTapOptions::ListenOnly,
                            vec![CGEventType::KeyDown],
                            |_proxy, _type, event| Some(event.to_owned()),
                        );

                        if test_tap.is_ok() {
                            app_info!("检测到辅助功能权限已授予！正在重新启动监听器...");
                            drop(test_tap);
                            // 重新启动监听器
                            start_listener(app_clone);
                            return;
                        } else {
                            app_debug!("辅助功能权限仍未授予，继续等待...");
                        }
                    }
                });
                return;
            }
        };

        app_debug!("正在创建 RunLoop Source...");
        let loop_source = tap
            .mach_port
            .create_runloop_source(0)
            .expect("Failed to create runloop source");
        let current_loop = CFRunLoop::get_current();
        current_loop.add_source(&loop_source, unsafe { kCFRunLoopCommonModes });

        app_debug!("正在启用 EventTap...");
        tap.enable();

        app_info!("热键监听器已启动并运行中，等待键盘事件...");
        CFRunLoop::run_current();

        // 如果 RunLoop 退出，记录日志
        app_error!("CFRunLoop 已退出！热键监听可能已停止工作");
    });
}

#[cfg(target_os = "windows")]
pub fn start_listener(app: tauri::AppHandle) {
    app_info!("正在启动 Windows 热键监听器...");
    let state = Arc::new(Mutex::new(DoubleTapState::default()));
    let app_handle = app.clone();

    std::thread::spawn(move || {
        app_info!("Windows 热键监听线程已启动");
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
                            app_debug!("距离上次按键时间: {:?}", diff);
                            diff <= Duration::from_millis(450)
                        });

                        if !is_double {
                            app_debug!("检测到第一次 Ctrl+C");
                            guard.last_c_press = Some(now);
                        } else {
                            app_info!("检测到双击 Ctrl+C！触发悬浮翻译...");
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
            app_error!("Windows 热键监听出错: {:?}", e);
        }
    });
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn start_listener(_app: tauri::AppHandle) {
    app_info!("热键监听在此操作系统上尚未实现 (Linux/Other)");
}
