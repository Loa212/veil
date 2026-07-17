//! The complete Tauri command surface. Every command is registered exactly once
//! in `generate_handler!` (lib.rs) — there is no second registration site.

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::auth::AuthOutcome;
use crate::settings::{self, Settings};
use crate::state::{self, State};

// ── State machine ───────────────────────────────────────────────────────────

/// Raise the lock overlay now (menubar "Lock now" / hotkey). Idle -> Presenting.
#[tauri::command]
pub fn lock_now(app: AppHandle) {
    request_lock(&app);
}

/// Shared lock entry point for the command, tray, and hotkey. Refuses to lock
/// until a PIN is configured (otherwise there'd be no way to unlock); instead it
/// surfaces the first-run setup.
pub fn request_lock(app: &AppHandle) {
    if !crate::auth::is_pin_configured() {
        log::warn!("lock requested before PIN setup → opening first-run");
        open_first_run_window(app);
        return;
    }
    state::transition(app, State::Presenting);
}

#[tauri::command]
pub fn get_state(app: AppHandle) -> State {
    state::current(&app)
}

/// Leave the Frozen state after a failed-auth lock. Frozen -> Idle.
#[tauri::command]
pub fn resume(app: AppHandle) {
    if state::current(&app) == State::Frozen {
        state::transition(&app, State::Idle);
    }
}

// ── Auth ──────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn authenticate_touchid(app: AppHandle) -> AuthOutcome {
    let outcome = crate::auth::authenticate(&app, "unlock Veil");
    log::info!("touchid outcome: {outcome:?}");
    if matches!(outcome, AuthOutcome::Success) {
        unlock(&app);
    }
    outcome
}

#[tauri::command]
pub fn verify_pin(app: AppHandle, pin: String) -> bool {
    let ok = crate::auth::verify_pin(&pin);
    if ok {
        unlock(&app);
    }
    ok
}

/// Successful auth from the overlay: tear it down and go Idle (no re-arm).
fn unlock(app: &AppHandle) {
    let current = state::current(app);
    log::info!("unlock requested (current state: {current:?})");
    // Frozen normally clears through the macOS screen-unlock notification, but
    // that distributed notification is not guaranteed. Accepting auth here is
    // the recovery path that ensures a missed notification cannot strand an
    // overlay above the desktop.
    if matches!(current, State::Presenting | State::Frozen) {
        state::transition(app, State::Idle);
    }
}

#[tauri::command]
pub fn auth_failed(app: AppHandle) -> Result<(), String> {
    // Only freeze after the native lock was actually raised. Freezing on an API
    // failure disables normal overlay auth and used to create an unrecoverable
    // lock screen.
    crate::lock::lock_screen().map_err(|e| {
        log::error!("native lock failed; keeping overlay auth active: {e}");
        e
    })?;
    state::transition(&app, State::Frozen);
    Ok(())
}

// ── PIN setup ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn is_pin_configured() -> bool {
    crate::auth::is_pin_configured()
}

/// First-run: store the chosen PIN (hashed) in the Keychain.
#[tauri::command]
pub fn setup_pin(pin: String) -> Result<(), String> {
    validate_pin(&pin)?;
    crate::auth::set_pin(&pin)
}

/// Settings: change the PIN. Authorized by the current PIN OR Touch ID — the
/// frontend runs Touch ID and passes `touchIdOk: true` if it succeeded.
#[tauri::command]
pub fn change_pin(
    app: AppHandle,
    new_pin: String,
    current_pin: Option<String>,
    touch_id_ok: Option<bool>,
) -> Result<bool, String> {
    let _ = &app;
    let authorized = touch_id_ok.unwrap_or(false)
        || current_pin
            .as_deref()
            .map(crate::auth::verify_pin)
            .unwrap_or(false);
    if !authorized {
        return Ok(false);
    }
    validate_pin(&new_pin)?;
    crate::auth::set_pin(&new_pin)?;
    Ok(true)
}

fn validate_pin(pin: &str) -> Result<(), String> {
    if pin.len() < 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("PIN must be at least 4 digits".into());
    }
    Ok(())
}

// ── Settings ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn load_settings(app: AppHandle) -> Settings {
    settings::load(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings::save(&app, &settings)?;
    // Apply the login-item toggle to match the saved setting.
    if let Err(e) = apply_launch_at_login(&app, settings.launch_at_login) {
        log::warn!("failed to apply launch-at-login: {e}");
    }
    let mutex = app.state::<std::sync::Mutex<state::AppState>>();
    mutex.lock().unwrap().settings = settings;
    state::sync_power(&app);
    let _ = app.emit("settings-updated", ());
    Ok(())
}

#[tauri::command]
pub fn pick_background(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .add_filter("Images", &["png", "jpg", "jpeg", "heic", "webp", "gif"])
        .blocking_pick_file();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<(), String> {
    apply_launch_at_login(&app, enabled)
}

/// Sync the macOS login item to `enabled`. Shared so `save_settings` can apply
/// the toggle too.
pub fn apply_launch_at_login(app: &AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    if enabled {
        mgr.enable().map_err(|e| format!("enable autostart: {e}"))
    } else {
        mgr.disable().map_err(|e| format!("disable autostart: {e}"))
    }
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
        .inner_size(720.0, 560.0)
        .min_inner_size(640.0, 480.0)
        .resizable(true)
        .center()
        .build();
    if let Err(e) = result {
        log::error!("failed to open settings window: {e}");
    }
}

/// Open the first-run setup window (no PIN configured yet). Closing it after a
/// successful setup is the frontend's job.
pub fn open_first_run_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("first-run") {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }
    let result = WebviewWindowBuilder::new(app, "first-run", WebviewUrl::App("index.html".into()))
        .title("Welcome to Veil")
        .inner_size(560.0, 680.0)
        .resizable(false)
        .center()
        .build();
    if let Err(e) = result {
        log::error!("failed to open first-run window: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::validate_pin;

    #[test]
    fn accepts_supported_pin_lengths() {
        assert!(validate_pin("1234").is_ok());
        assert!(validate_pin("123456").is_ok());
        assert!(validate_pin("12345678901234567890").is_ok());
    }

    #[test]
    fn rejects_unsupported_or_non_numeric_pins() {
        assert!(validate_pin("123").is_err());
        assert!(validate_pin("12a4").is_err());
    }
}
