//! Veil — a macOS soft lockscreen. This is the thin app entry point: it wires
//! plugins, manages the state machine, builds the tray, installs the focus
//! watcher, and registers the command surface. Business logic lives in modules.

mod auth;
mod commands;
mod focus;
mod keychain;
mod lock;
mod overlay;
mod power;
mod recovery;
mod settings;
mod state;
mod tray;

use std::sync::Mutex;

use tauri::{Listener, Manager};

use crate::state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
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

            // Tray + focus watcher.
            let tray = tray::build(&handle)?;
            app.manage(tray);
            focus::install(&handle);

            // Keep the tray in sync with the state machine.
            let h = handle.clone();
            app.listen("state-changed", move |_| tray::refresh(&h));
            tray::refresh(&handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::arm,
            commands::disarm,
            commands::get_state,
            commands::resume,
            commands::present_overlay,
            commands::dismiss_overlay,
            commands::authenticate_touchid,
            commands::verify_pin,
            commands::verify_recovery,
            commands::auth_failed,
            commands::is_pin_configured,
            commands::setup_pin,
            commands::change_pin,
            commands::generate_recovery,
            commands::regenerate_recovery,
            commands::load_settings,
            commands::save_settings,
            commands::pick_background,
            commands::set_launch_at_login,
            commands::open_settings_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Veil");
}
