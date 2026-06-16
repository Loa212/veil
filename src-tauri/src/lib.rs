//! Veil — a macOS soft lockscreen. This is the thin app entry point: it wires
//! plugins, manages the state machine, builds the tray, registers the global
//! lock hotkey, and registers the command surface. Business logic lives in
//! modules.

mod auth;
mod commands;
mod keychain;
mod lock;
mod overlay;
mod power;
mod screen;
mod settings;
mod state;
mod tray;

use std::sync::Mutex;

use tauri::{Listener, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::state::AppState;

/// Global hotkey that raises the lock overlay from anywhere: Cmd+Ctrl+L.
fn lock_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::SUPER | Modifiers::CONTROL), Code::KeyL)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, sc, event| {
                    if event.state() == ShortcutState::Pressed && sc == &lock_shortcut() {
                        commands::request_lock(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            // Menubar-only app: no Dock icon, no app-switcher entry. Must run
            // before we take an immutable handle, since it needs `&mut App`.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle().clone();

            // Load persisted settings and seed the managed state.
            let loaded = settings::load(&handle);
            app.manage(Mutex::new(AppState::new(loaded)));

            // Register the global lock hotkey (Cmd+Ctrl+L).
            if let Err(e) = app.global_shortcut().register(lock_shortcut()) {
                log::error!("failed to register lock hotkey: {e}");
            }

            // Auto-clear Frozen once macOS is unlocked again.
            screen::install(&handle);

            // Tray.
            let tray = tray::build(&handle)?;
            app.manage(tray);

            // Keep the tray in sync with the state machine.
            let h = handle.clone();
            app.listen("state-changed", move |_| tray::refresh(&h));
            tray::refresh(&handle);

            // First run: no PIN yet → open the setup window.
            if !auth::is_pin_configured() {
                commands::open_first_run_window(&handle);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::lock_now,
            commands::get_state,
            commands::resume,
            commands::authenticate_touchid,
            commands::verify_pin,
            commands::auth_failed,
            commands::is_pin_configured,
            commands::setup_pin,
            commands::change_pin,
            commands::load_settings,
            commands::save_settings,
            commands::pick_background,
            commands::set_launch_at_login,
            commands::open_settings_window,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Veil")
        .run(|_app, event| {
            // Menubar app: keep running in the tray when all windows close
            // (e.g. after the overlay is dismissed on unlock). Explicit
            // `app.exit(0)` from the tray Quit item still exits.
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
