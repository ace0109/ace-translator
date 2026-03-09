use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

pub fn read_clipboard(app: &AppHandle) -> Result<String, String> {
    // Use the clipboard plugin to read text
    // Note: ensure tauri-plugin-clipboard-manager is initialized in lib.rs
    app.clipboard().read_text().map_err(|e| e.to_string())
}
