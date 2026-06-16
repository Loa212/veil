//! Overlay manager: spawns one borderless screen-saver-level window per display
//! on `Armed -> Presenting`, tears them all down on leaving Presenting.

mod nswindow;
mod windows;

use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use crate::state::AppState;

/// Spawn overlay windows across every display. Wired into the
/// `Armed -> Presenting` edge (Phase 3).
pub fn present(app: &AppHandle) {
    let labels = match windows::spawn_all(app) {
        Ok(labels) => labels,
        Err(e) => {
            log::error!("failed to present overlay: {e}");
            Vec::new()
        }
    };
    let state = app.state::<Mutex<AppState>>();
    state.lock().unwrap().overlay_labels = labels;
}

/// Close all overlay windows and clear tracked labels.
pub fn dismiss(app: &AppHandle) {
    let state = app.state::<Mutex<AppState>>();
    let labels = {
        let mut guard = state.lock().unwrap();
        std::mem::take(&mut guard.overlay_labels)
    };
    log::info!("dismissing {} overlay window(s)", labels.len());
    for label in labels {
        match app.get_webview_window(&label) {
            Some(win) => {
                if let Err(e) = win.close() {
                    log::warn!("failed to close overlay {label}: {e}");
                }
            }
            None => log::warn!("overlay {label} not found at dismiss time"),
        }
    }
}
