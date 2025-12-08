use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[allow(dead_code)]
pub fn simulate_copy() -> Result<(), String> {
    // Initialize Enigo
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    // Determine modifier key based on OS
    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;

    // Simulate Ctrl+C (or Cmd+C)
    // Press Modifier
    enigo
        .key(modifier, Direction::Press)
        .map_err(|e| e.to_string())?;

    // Click C
    // Note: Key::C matches 'c'
    enigo
        .key(Key::Unicode('c'), Direction::Click)
        .map_err(|e| e.to_string())?;

    // Release Modifier
    enigo
        .key(modifier, Direction::Release)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn read_clipboard(app: &AppHandle) -> Result<String, String> {
    // Use the clipboard plugin to read text
    // Note: ensure tauri-plugin-clipboard-manager is initialized in lib.rs
    app.clipboard().read_text().map_err(|e| e.to_string())
}
