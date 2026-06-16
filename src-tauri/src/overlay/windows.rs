//! Enumerate displays and spawn one overlay `WebviewWindow` per screen.
//!
//! Phase 1 stub: returns no windows. Phase 3 implements `NSScreen::screens()`
//! enumeration + `WebviewWindowBuilder` per display, then applies the
//! screen-saver window level via `super::nswindow`.

use tauri::AppHandle;

pub fn spawn_all(_app: &AppHandle) -> Result<Vec<String>, String> {
    // Phase 3: enumerate NSScreen::screens(), build one overlay-{i} window each.
    Ok(Vec::new())
}
