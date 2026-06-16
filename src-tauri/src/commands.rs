//! The complete Tauri command surface. Every command is registered exactly once
//! in `generate_handler!` (lib.rs) — there is no second registration site.

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::auth::AuthOutcome;
use crate::settings::{self, Settings};
use crate::state::{self, State};

// ── State machine ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn arm(app: AppHandle) {
    state::transition(&app, State::Armed);
}

#[tauri::command]
pub fn disarm(app: AppHandle) {
    state::transition(&app, State::Idle);
}

#[tauri::command]
pub fn get_state(app: AppHandle) -> State {
    state::current(&app)
}

#[tauri::command]
pub fn resume(app: AppHandle) {
    if state::current(&app) == State::Frozen {
        state::transition(&app, State::Armed);
    }
}

// ── Overlay (dev/testing; normal path is the focus watcher) ──────────────────

#[tauri::command]
pub fn present_overlay(app: AppHandle) {
    state::transition(&app, State::Presenting);
}

#[tauri::command]
pub fn dismiss_overlay(app: AppHandle) {
    if state::current(&app) == State::Presenting {
        state::transition(&app, State::Armed);
    }
}

// ── Auth ──────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn authenticate_touchid() -> AuthOutcome {
    crate::auth::authenticate("unlock Veil")
}

#[tauri::command]
pub fn verify_pin(_pin: String) -> bool {
    // Phase 4/5: compare against the Keychain-stored hash, transition on success.
    false
}

#[tauri::command]
pub fn verify_recovery(_code: String) -> bool {
    // Phase 4/5.
    false
}

#[tauri::command]
pub fn auth_failed(app: AppHandle) {
    // Phase 4: trigger the native lock, then freeze.
    if let Err(e) = crate::lock::lock_screen() {
        log::error!("native lock failed: {e}");
    }
    state::transition(&app, State::Frozen);
}

// ── PIN / recovery setup ──────────────────────────────────────────────────────

#[tauri::command]
pub fn is_pin_configured() -> bool {
    // Phase 5: check the Keychain.
    false
}

#[tauri::command]
pub fn setup_pin(_pin: String) -> Result<(), String> {
    // Phase 5.
    Err("not implemented".into())
}

#[tauri::command]
pub fn change_pin(_current_pin: String, _new_pin: String) -> Result<bool, String> {
    // Phase 5/7.
    Err("not implemented".into())
}

#[tauri::command]
pub fn generate_recovery() -> Result<String, String> {
    // Phase 5.
    Err("not implemented".into())
}

#[tauri::command]
pub fn regenerate_recovery() -> Result<String, String> {
    // Phase 7.
    Err("not implemented".into())
}

// ── Settings ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn load_settings(app: AppHandle) -> Settings {
    settings::load(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings::save(&app, &settings)?;
    let mutex = app.state::<std::sync::Mutex<state::AppState>>();
    mutex.lock().unwrap().settings = settings;
    let _ = app.emit("settings-updated", ());
    Ok(())
}

#[tauri::command]
pub fn pick_background() -> Result<Option<String>, String> {
    // Phase 7: dialog plugin file picker.
    Ok(None)
}

#[tauri::command]
pub fn set_launch_at_login(_enabled: bool) -> Result<(), String> {
    // Phase 8.
    Err("not implemented".into())
}

// ── Windows ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn open_settings_window(app: AppHandle) {
    open_settings_window_impl(&app);
}

/// Shared impl so the tray can open settings without going through IPC.
pub fn open_settings_window_impl(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }
    let result = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("Veil — Settings")
        .inner_size(560.0, 640.0)
        .resizable(true)
        .center()
        .build();
    if let Err(e) = result {
        log::error!("failed to open settings window: {e}");
    }
}
