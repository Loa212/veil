//! Raw-NSWindow manipulation for overlay windows: set the screen-saver window
//! level and the all-spaces collection behavior so the overlay covers the menu
//! bar and Dock and follows the user across Spaces.
//!
//! Phase 1 stub. Phase 3 implements the `ns_window() -> usize ->
//! run_on_main_thread` idiom to call `setLevel(1000)` / `setCollectionBehavior`.

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn elevate(_window: &tauri::WebviewWindow) -> Result<(), String> {
    // Phase 3.
    Ok(())
}
