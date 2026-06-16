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

    let ptr = window.ns_window().map_err(|e| format!("ns_window: {e}"))? as usize;
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
            // Force the window to the front even when Veil isn't the active app
            // — otherwise an overlay on a display whose Space isn't focused
            // won't paint until that display is clicked.
            ns_window.orderFrontRegardless();
        })
        .map_err(|e| format!("run_on_main_thread: {e}"))
}

/// Activate Veil (ignoring other apps) so macOS lets its `CanJoinAllSpaces`
/// overlay windows composite onto every display's current Space immediately —
/// otherwise a window on a display whose Space isn't focused won't paint until
/// that display is clicked. Run via any overlay window's main thread.
#[cfg(target_os = "macos")]
pub fn activate_app(window: &tauri::WebviewWindow) -> Result<(), String> {
    use objc2_app_kit::NSApplication;
    use objc2_foundation::MainThreadMarker;

    window
        .run_on_main_thread(|| {
            // SAFETY: we are on the main thread inside this closure.
            let mtm = unsafe { MainThreadMarker::new_unchecked() };
            let app = NSApplication::sharedApplication(mtm);
            #[allow(deprecated)]
            app.activateIgnoringOtherApps(true);
        })
        .map_err(|e| format!("run_on_main_thread (activate): {e}"))
}

#[cfg(not(target_os = "macos"))]
pub fn elevate(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn activate_app(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}
