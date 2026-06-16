//! Enumerate displays and spawn one borderless overlay window per screen.
//!
//! We use Tauri's `available_monitors()` (which already returns physical
//! position + size in the global coordinate space) rather than enumerating
//! `NSScreen` ourselves — this sidesteps the Cocoa bottom-left coordinate
//! conversion and the main-thread marker. Each window is positioned/sized in
//! physical pixels, then elevated to screen-saver level via `super::nswindow`.

use tauri::{AppHandle, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

use super::nswindow;

/// Spawn an overlay window on every display. Returns the labels of the windows
/// created (so the manager can tear them down later).
pub fn spawn_all(app: &AppHandle) -> Result<Vec<String>, String> {
    let monitors = app
        .available_monitors()
        .map_err(|e| format!("available_monitors: {e}"))?;

    // Identify the primary monitor so the overlay there renders the auth prompt.
    let primary = app
        .primary_monitor()
        .ok()
        .flatten()
        .and_then(|m| m.name().cloned());

    let mut labels = Vec::with_capacity(monitors.len());

    for (i, monitor) in monitors.iter().enumerate() {
        let pos = *monitor.position();
        let size = *monitor.size();
        let is_primary = match (&primary, monitor.name()) {
            (Some(p), Some(n)) => p == n,
            // Fall back to "first monitor is primary" when names are missing.
            _ => i == 0,
        };

        let label = format!("overlay-{i}");
        let url = format!(
            "index.html?role=overlay&idx={i}&primary={is_primary}",
        );

        let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title("")
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .resizable(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .focused(is_primary)
            .build()
            .map_err(|e| format!("build {label}: {e}"))?;

        // Position/size in physical pixels to avoid logical/physical ambiguity
        // across mixed-DPI displays.
        window
            .set_position(PhysicalPosition::new(pos.x, pos.y))
            .map_err(|e| format!("set_position {label}: {e}"))?;
        window
            .set_size(PhysicalSize::new(size.width, size.height))
            .map_err(|e| format!("set_size {label}: {e}"))?;

        nswindow::elevate(&window)?;

        labels.push(label);
    }

    log::info!("presented {} overlay window(s)", labels.len());
    Ok(labels)
}
