use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    alt_down: bool,  // For Alt/Option key
}

// Key Codes for macOS
#[cfg(target_os = "macos")]
const KEY_C: CGKeyCode = 0x08;
#[cfg(target_os = "macos")]
const KEY_SPACE: CGKeyCode = 0x31;

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

// Alt+Space 快捷键处理：屏幕居中显示主窗口
async fn handle_alt_space(app: tauri::AppHandle) {
    // 检查是否启用
    {
        let state: tauri::State<AppState> = app.state();
        let enabled = state.hotkey_alt_space_enabled.lock().map(|g| *g).unwrap_or(true);
        if !enabled {
            app_debug!("Alt+Space 快捷键已禁用，跳过");
            return;
        }
    }

    app_info!("检测到 Alt+Space 快捷键！正在显示主窗口...");

    if let Some(window) = app.get_webview_window("main") {
        // 使用鼠标所在屏幕居中显示，失败则回退到当前屏幕
        if let Err(e) = crate::commands::system::center_window_on_active_screen(&window) {
            app_error!("窗口居中失败: {}", e);
            let _ = crate::commands::system::center_window_on_screen(&window);
        }

        match window.show() {
            Ok(_) => app_info!("主窗口显示成功"),
            Err(e) => app_error!("主窗口显示失败: {}", e),
        }

        match window.set_focus() {
            Ok(_) => app_debug!("主窗口获得焦点"),
            Err(e) => app_error!("主窗口获取焦点失败: {}", e),
        }
    } else {
        app_error!("无法获取主窗口");
    }
}

// Platform-agnostic handle_double_copy
async fn handle_double_copy(app: tauri::AppHandle) {
    // 检查是否启用
    {
        let state: tauri::State<AppState> = app.state();
        let enabled = state.hotkey_double_copy_enabled.lock().map(|g| *g).unwrap_or(true);
        if !enabled {
            app_debug!("双击复制翻译已禁用，跳过");
            return;
        }
    }

    app_info!("检测到双击复制！开始处理主窗口显示...");
    sleep(Duration::from_millis(100)).await; // Give system time to update clipboard

    app_debug!("正在读取剪贴板内容...");
    match clipboard::read_clipboard(&app) {
        Ok(text) => {
            app_info!("剪贴板内容读取成功，长度: {} 字符", text.len());
            if text.trim().is_empty() {
                app_info!("跳过：剪贴板内容为空或仅包含空白字符");
                return;
            }

            app_debug!("正在获取主窗口...");
            if let Some(window) = app.get_webview_window("main") {
                app_info!("成功获取到主窗口");

                // 未固定时固定显示在屏幕顶部 10% 处居中
                let pinned = {
                    let state: tauri::State<AppState> = app.state();
                    state.main_pinned.lock().map(|g| *g).unwrap_or(false)
                };

                let loading = {
                    let state: tauri::State<AppState> = app.state();
                    state.main_loading.lock().map(|g| *g).unwrap_or(false)
                };

                app_debug!("主窗口状态 - 固定: {}, 加载中: {}", pinned, loading);

                if !pinned && !loading {
                    if let Err(e) = crate::commands::system::center_window_on_active_screen(&window) {
                        app_error!("窗口居中失败: {}", e);
                        let _ = crate::commands::system::center_window_on_screen(&window);
                    }
                    app_info!("主窗口固定显示在当前屏幕顶部 5% 处居中");
                } else {
                    app_info!("主窗口已固定或加载中，保持当前位置");
                }

                app_debug!("正在显示主窗口...");
                match window.show() {
                    Ok(_) => app_info!("主窗口显示成功"),
                    Err(e) => app_error!("主窗口显示失败: {}", e),
                }

                match window.set_focus() {
                    Ok(_) => app_debug!("主窗口获得焦点"),
                    Err(e) => app_error!("主窗口获取焦点失败: {}", e),
                }

                sleep(Duration::from_millis(50)).await;

                app_debug!("正在发送 main-show 事件...");
                match window.emit("main-show", text.clone()) {
                    Ok(_) => app_info!("main-show 事件发送成功，文本长度: {}", text.len()),
                    Err(e) => app_error!("main-show 事件发送失败: {}", e),
                }
            } else {
                app_error!("无法获取主窗口！窗口可能未创建或已销毁");
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
                                "KeyDown: keycode={}, meta={}, ctrl={}, alt={}",
                                key_code, guard.meta_down, guard.ctrl_down, guard.alt_down
                            ));

                            // Alt/Option + Space 快捷键
                            if key_code == KEY_SPACE && guard.alt_down {
                                crate::services::logger::LOGGER.info("检测到 Option+Space 快捷键！");
                                let app_clone = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    handle_alt_space(app_clone).await;
                                });
                            }
                            // Cmd/Ctrl + C 双击检测
                            else if key_code == KEY_C {
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
                            let old_alt = guard.alt_down;

                            guard.ctrl_down = flags.contains(CGEventFlags::CGEventFlagControl);
                            guard.meta_down = flags.contains(CGEventFlags::CGEventFlagCommand);
                            guard.alt_down = flags.contains(CGEventFlags::CGEventFlagAlternate);

                            // 只在状态变化时记录
                            if old_meta != guard.meta_down || old_ctrl != guard.ctrl_down || old_alt != guard.alt_down {
                                crate::services::logger::LOGGER.debug(&format!(
                                    "修饰键状态变化: meta={}, ctrl={}, alt={}",
                                    guard.meta_down, guard.ctrl_down, guard.alt_down
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
                    } else if matches!(key, Key::Alt | Key::AltGr) {
                        guard.alt_down = true;
                    } else if key == Key::Space && guard.alt_down {
                        // Alt + Space 快捷键
                        app_info!("检测到 Alt+Space 快捷键！");
                        let app_clone = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_alt_space(app_clone).await;
                        });
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
                    } else if matches!(key, Key::Alt | Key::AltGr) {
                        guard.alt_down = false;
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
