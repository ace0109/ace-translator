/// macOS 专用：让窗口在被激活时自动移动到当前桌面，避免切换 Space。
/// 其他平台为 no-op。
pub fn ensure_window_on_current_space(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
        use cocoa::base::id;

        match window.ns_window() {
            Ok(ns_window_ptr) => unsafe {
                let ns_window: id = ns_window_ptr as id;
                let current = ns_window.collectionBehavior();
                let desired = current
                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorMoveToActiveSpace;

                // 仅在需要时更新，避免重复调用。
                if current != desired {
                    ns_window.setCollectionBehavior_(desired);
                    crate::app_debug!(
                        "已为窗口 {} 启用 MoveToActiveSpace，当前行为掩码: {:?}",
                        window.label(),
                        desired
                    );
                }
            },
            Err(e) => {
                crate::app_error!(
                    "获取窗口 {} 的 NSWindow 句柄失败，无法设置桌面行为: {}",
                    window.label(),
                    e
                );
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
    }
}
