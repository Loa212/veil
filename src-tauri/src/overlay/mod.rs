//! Overlay manager: spawns one borderless screen-saver-level window per display
//! on `Armed -> Presenting`, tears them all down on leaving Presenting.

mod nswindow;
mod windows;

use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use crate::state::AppState;

/// Spawn overlay windows across every display. Presentation is transactional:
/// `spawn_all` removes partial windows before returning an error.
pub fn present(app: &AppHandle) -> Result<(), String> {
    let labels = windows::spawn_all(app)?;
    let state = app.state::<Mutex<AppState>>();
    state.lock().unwrap().overlay_labels = labels;
    Ok(())
}

fn is_overlay_label(label: &str) -> bool {
    label
        .strip_prefix("overlay-")
        .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
}

/// Destroy all overlay windows and clear tracked labels. Discovering windows by
/// prefix as well as by the tracking list is deliberate: older/partial failures
/// may have created a window before its label was committed to managed state.
pub fn dismiss(app: &AppHandle) {
    let state = app.state::<Mutex<AppState>>();
    let labels = {
        let mut guard = state.lock().unwrap();
        std::mem::take(&mut guard.overlay_labels)
    };
    let mut windows = app
        .webview_windows()
        .into_iter()
        .filter(|(label, _)| is_overlay_label(label))
        .collect::<Vec<_>>();

    // Include tracked windows even if discovery semantics change in a future
    // Tauri version, while avoiding duplicates.
    for label in labels {
        if windows.iter().any(|(existing, _)| existing == &label) {
            continue;
        }
        if let Some(window) = app.get_webview_window(&label) {
            windows.push((label, window));
        }
    }

    log::info!("dismissing {} overlay window(s)", windows.len());
    for (label, window) in windows {
        // Hide first so even a native destroy failure cannot leave a
        // screen-saver-level surface trapping the user.
        if let Err(e) = window.hide() {
            log::warn!("failed to hide overlay {label}: {e}");
        }
        if let Err(e) = window.destroy() {
            log::error!("failed to destroy overlay {label}: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_overlay_label;

    #[test]
    fn recognizes_only_managed_overlay_labels() {
        assert!(is_overlay_label("overlay-0"));
        assert!(is_overlay_label("overlay-12"));
        assert!(!is_overlay_label("overlay-"));
        assert!(!is_overlay_label("overlay-settings"));
        assert!(!is_overlay_label("settings"));
    }
}
