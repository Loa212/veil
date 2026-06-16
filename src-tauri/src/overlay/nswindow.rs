//! Raw-NSWindow manipulation for overlay windows.
//!
//! Tauri's own `set_level` / `set_collection_behavior` hit a fullscreen-release
//! bug (tauri#5566), so we reach the underlying `NSWindow` directly and call
//! AppKit. This is the linchpin that makes the overlay cover the menu bar and
//! Dock and follow the user across Spaces.

/// `NSScreenSaverWindowLevel` == `kCGScreenSaverWindowLevel` == 1000. Covers the
/// menu bar and Dock (anything below the system screensaver itself).
#[cfg(target_os = "macos")]
const SCREEN_SAVER_LEVEL: isize = 1000;

/// Elevate an overlay window to screen-saver level with all-spaces, stationary,
/// fullscreen-auxiliary collection behavior, and keep it capturing mouse events
/// (we want clicks to reveal the auth prompt).
#[cfg(target_os = "macos")]
pub fn elevate(window: &tauri::WebviewWindow) -> Result<(), String> {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    let ptr = window
        .ns_window()
        .map_err(|e| format!("ns_window: {e}"))? as usize;
    if ptr == 0 {
        return Err("ns_window pointer is null".into());
    }

    window
        .run_on_main_thread(move || {
            // SAFETY: `ptr` is the live NSWindow backing this Tauri window; we
            // only touch it on the main thread (AppKit requirement) and the
            // window outlives this synchronous closure.
            let ns_window: &NSWindow = unsafe { &*(ptr as *const NSWindow) };
            ns_window.setLevel(SCREEN_SAVER_LEVEL);
            ns_window.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::FullScreenAuxiliary
                    | NSWindowCollectionBehavior::Stationary,
            );
            ns_window.setIgnoresMouseEvents(false);
        })
        .map_err(|e| format!("run_on_main_thread: {e}"))
}

#[cfg(not(target_os = "macos"))]
pub fn elevate(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}
