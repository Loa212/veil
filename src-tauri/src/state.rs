//! Core state machine. Every state transition flows through `transition()`,
//! which validates the edge, runs side effects, and emits `state-changed`.
//!
//! Veil locks only on an explicit user action (menubar "Lock now" or the global
//! hotkey). After a successful unlock it returns to `Idle` — it does NOT re-lock
//! on focus loss. Hence there is no `Armed` state and no focus watcher.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::settings::Settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// Not locked. The resting state.
    Idle,
    /// Overlay is up, awaiting auth.
    Presenting,
    /// Auth failed/dismissed: the native lock was triggered and the overlay torn
    /// down. Stays here until the user resumes.
    Frozen,
}

/// App-wide state, managed as `Mutex<AppState>`.
///
/// A `std::sync::Mutex` (not tokio's) is correct here: transitions are
/// synchronous and short, and no `.await` is ever held across the lock.
pub struct AppState {
    pub state: State,
    /// IOKit power-assertion id held while the overlay is up (prevents idle
    /// sleep when `prevent_sleep` is enabled).
    pub power_assertion_id: Option<u32>,
    /// Labels of currently-spawned overlay windows.
    pub overlay_labels: Vec<String>,
    pub settings: Settings,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            state: State::Idle,
            power_assertion_id: None,
            overlay_labels: Vec::new(),
            settings,
        }
    }
}

/// Returns true if `from -> to` is a legal edge in the state machine.
///
/// ```text
///        ┌──────┐  lock now / hotkey   ┌────────────┐
///        │ Idle │ ───────────────────► │ Presenting │
///        └──────┘ ◄─────────────────── └────────────┘
///           ▲           auth ok          │ auth fail
///           │ resume                      ▼
///           │                         ┌────────┐
///           └──────────────────────── │ Frozen │
///                                      └────────┘
/// ```
/// `* -> Idle` is always legal (resume / disarm / quit).
pub fn is_legal(from: State, to: State) -> bool {
    use State::*;
    if to == Idle {
        return true;
    }
    matches!((from, to), (Idle, Presenting) | (Presenting, Frozen))
}

/// Read the current state.
pub fn current(app: &AppHandle) -> State {
    let state = app.state::<Mutex<AppState>>();
    let guard = state.lock().unwrap();
    guard.state
}

/// Attempt a transition. Illegal edges are no-ops (logged at warn). On a legal
/// edge: updates the state, runs side effects, and emits `state-changed`.
pub fn transition(app: &AppHandle, to: State) {
    let managed = app.state::<Mutex<AppState>>();
    let from = {
        let mut guard = managed.lock().unwrap();
        let from = guard.state;
        if from == to {
            return;
        }
        if !is_legal(from, to) {
            log::warn!("ignoring illegal transition {from:?} -> {to:?}");
            return;
        }
        guard.state = to;
        from
    };

    log::info!("state: {from:?} -> {to:?}");
    run_side_effects(app, from, to);

    if let Err(e) = app.emit("state-changed", to) {
        log::error!("failed to emit state-changed: {e}");
    }
}

/// Side effects attached to specific edges.
fn run_side_effects(app: &AppHandle, from: State, to: State) {
    use State::*;
    match (from, to) {
        // Lock now: raise the overlay + (optionally) prevent idle sleep.
        (Idle, Presenting) => {
            crate::overlay::present(app);
            acquire_power(app);
        }
        // Unlocked: tear the overlay down + release the sleep assertion.
        (Presenting, Idle) => {
            crate::overlay::dismiss(app);
            release_power(app);
        }
        // Failed auth -> Frozen: the native lock (SACLockScreenImmediate) is
        // already firing; KEEP our overlay up so the system lock covers it with
        // no desktop flash. Release our assertion (the OS lock owns sleep now).
        (Presenting, Frozen) => release_power(app),
        // Returned to Idle after macOS unlock: tear the overlay down.
        (Frozen, Idle) => crate::overlay::dismiss(app),
        _ => {}
    }
}

/// Acquire an idle-sleep assertion if `prevent_sleep` is enabled.
fn acquire_power(app: &AppHandle) {
    let managed = app.state::<Mutex<AppState>>();
    let mut guard = managed.lock().unwrap();
    if !guard.settings.prevent_sleep || guard.power_assertion_id.is_some() {
        return;
    }
    match crate::power::acquire() {
        Ok(id) => guard.power_assertion_id = Some(id),
        Err(e) => log::warn!("prevent-sleep assertion failed: {e}"),
    }
}

/// Release the idle-sleep assertion if one is held.
fn release_power(app: &AppHandle) {
    let managed = app.state::<Mutex<AppState>>();
    let id = managed.lock().unwrap().power_assertion_id.take();
    if let Some(id) = id {
        crate::power::release(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_edges() {
        use State::*;
        assert!(is_legal(Idle, Presenting));
        assert!(is_legal(Presenting, Frozen));
        // unlock / resume / disarm: anything -> Idle
        assert!(is_legal(Presenting, Idle));
        assert!(is_legal(Frozen, Idle));
    }

    #[test]
    fn illegal_edges() {
        use State::*;
        assert!(!is_legal(Idle, Frozen));
        assert!(!is_legal(Frozen, Presenting));
    }
}
