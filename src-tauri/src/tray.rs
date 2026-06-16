//! Menubar tray: arm / disarm / resume / status / settings / quit.
//!
//! The status line and the enabled/disabled state of arm/disarm/resume are
//! kept in sync with the state machine via `refresh()`, called on every
//! `state-changed`.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager};

use crate::state::{self, State};

/// Stable id used to look the tray up from `refresh()` (for tooltip updates).
const TRAY_ID: &str = "veil-tray";

pub struct TrayItems {
    pub status: MenuItem<tauri::Wry>,
    pub arm: MenuItem<tauri::Wry>,
    pub disarm: MenuItem<tauri::Wry>,
    pub resume: MenuItem<tauri::Wry>,
}

/// Build the tray icon + menu and register event handlers. Returns the tray so
/// the caller can keep it alive (dropping it removes the icon).
pub fn build(app: &AppHandle) -> tauri::Result<TrayIcon> {
    let status = MenuItem::with_id(app, "status", "Veil: Idle", false, None::<&str>)?;
    let arm = MenuItem::with_id(app, "arm", "Arm", true, None::<&str>)?;
    let disarm = MenuItem::with_id(app, "disarm", "Disarm", false, None::<&str>)?;
    let resume = MenuItem::with_id(app, "resume", "Resume", false, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Veil", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &status, &arm, &disarm, &resume, &sep1, &settings, &sep2, &quit,
        ],
    )?;

    app.manage(TrayItems {
        status,
        arm,
        disarm,
        resume,
    });

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(true)
        .tooltip("Veil")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(on_menu_event)
        .build(app)?;

    Ok(tray)
}

fn on_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "arm" => state::transition(app, State::Armed),
        "disarm" => state::transition(app, State::Idle),
        "resume" => state::transition(app, State::Armed),
        "settings" => crate::commands::open_settings_window_impl(app),
        "quit" => app.exit(0),
        _ => {}
    }
}

/// Sync the tray's status text and item-enabled flags to the current state.
/// Called on every `state-changed`.
pub fn refresh(app: &AppHandle) {
    let Some(items) = app.try_state::<TrayItems>() else {
        return;
    };
    let state = state::current(app);
    let label = match state {
        State::Idle => "Veil: Idle",
        State::Armed => "Veil: Armed",
        State::Presenting => "Veil: Presenting",
        State::Frozen => "Veil: Frozen",
    };
    let _ = items.status.set_text(label);
    let _ = items.arm.set_enabled(state == State::Idle);
    let _ = items.disarm.set_enabled(state != State::Idle);
    let _ = items.resume.set_enabled(state == State::Frozen);

    // Reflect state in the hover tooltip so it's glanceable without opening the
    // menu.
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(label));
    }
}
