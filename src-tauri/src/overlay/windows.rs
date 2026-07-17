//! Enumerate displays and spawn one borderless overlay window per screen.
//!
//! `available_monitors()` reports each display's **physical** position + size in
//! a global coordinate space. The `WebviewWindowBuilder` takes **logical**
//! coordinates, so we convert per-monitor (`logical = physical / scale`) and
//! build each window already at its final geometry — building at the wrong size
//! and resizing afterwards leaves a stale small window on any display whose
//! Space isn't currently focused. We then elevate to screen-saver level and
//! activate the app so every overlay composites across Spaces at once.

use tauri::{AppHandle, LogicalPosition, LogicalSize, WebviewUrl, WebviewWindowBuilder};

use super::nswindow;

/// Hide and destroy windows created by an incomplete presentation. Hiding first
/// is the safety-critical operation: it guarantees the user is not trapped even
/// if native teardown reports an error.
fn rollback(windows: &[tauri::WebviewWindow]) {
    for window in windows.iter().rev() {
        if let Err(e) = window.hide() {
            log::warn!("failed to hide partial overlay {}: {e}", window.label());
        }
        if let Err(e) = window.destroy() {
            log::error!("failed to destroy partial overlay {}: {e}", window.label());
        }
    }
}

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
    let mut first_window: Option<tauri::WebviewWindow> = None;
    let mut created = Vec::with_capacity(monitors.len());

    for (i, monitor) in monitors.iter().enumerate() {
        let pos = *monitor.position();
        let size = *monitor.size();
        let scale = monitor.scale_factor();
        let is_primary = match (&primary, monitor.name()) {
            (Some(p), Some(n)) => p == n,
            // Fall back to "first monitor is primary" when names are missing.
            _ => i == 0,
        };

        // Physical (global) -> logical, per-monitor.
        let lx = pos.x as f64 / scale;
        let ly = pos.y as f64 / scale;
        let lw = size.width as f64 / scale;
        let lh = size.height as f64 / scale;

        log::info!(
            "monitor {i} ({:?}): pos={:?} size={:?} scale={scale} -> logical {lx},{ly} {lw}x{lh} primary={is_primary}",
            monitor.name(),
            pos,
            size,
        );

        let label = format!("overlay-{i}");
        let url = format!("index.html?role=overlay&idx={i}&primary={is_primary}");

        let window = match WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title("")
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .resizable(false)
            .skip_taskbar(true)
            .always_on_top(true)
            // Build at the final geometry so the window never has a wrong-size
            // state on an unfocused display's Space.
            .position(lx, ly)
            .inner_size(lw, lh)
            .focused(is_primary)
            .build()
        {
            Ok(window) => window,
            Err(e) => {
                rollback(&created);
                return Err(format!("build {label}: {e}"));
            }
        };
        created.push(window.clone());

        // Re-assert exact logical geometry (the builder can be nudged by the WM).
        let _ = window.set_position(LogicalPosition::new(lx, ly));
        let _ = window.set_size(LogicalSize::new(lw, lh));

        if let Err(e) = nswindow::elevate(&window) {
            rollback(&created);
            return Err(format!("elevate {label}: {e}"));
        }
        let _ = window.show();

        if first_window.is_none() {
            first_window = Some(window);
        }
        labels.push(label);
    }

    // Activate Veil once so its all-Spaces overlay windows paint on every
    // display immediately rather than on first click of each display.
    if let Some(w) = &first_window {
        if let Err(e) = nswindow::activate_app(w) {
            rollback(&created);
            return Err(format!("activate overlays: {e}"));
        }
    }

    log::info!("presented {} overlay window(s)", labels.len());
    Ok(labels)
}
