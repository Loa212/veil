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
    /// IOKit power-assertion id held for the entire Veil lock session (including
    /// the native-lock fallback) when `prevent_sleep` is enabled.
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
    if let Err(e) = run_side_effects(app, from, to) {
        log::error!("state transition {from:?} -> {to:?} failed: {e}");

        // Overlay presentation is the only fallible transition side effect. It
        // is all-or-nothing: return to the previous state and defensively remove
        // any window that may have been created before the failure.
        let mut guard = managed.lock().unwrap();
        if guard.state == to {
            guard.state = from;
        }
        drop(guard);
        crate::overlay::dismiss(app);
        sync_power(app);
        return;
    }

    if let Err(e) = app.emit("state-changed", to) {
        log::error!("failed to emit state-changed: {e}");
    }
}

/// Side effects attached to specific edges.
fn run_side_effects(app: &AppHandle, from: State, to: State) -> Result<(), String> {
    use State::*;
    match (from, to) {
        // Lock now: raise the overlay + (optionally) prevent idle sleep.
        (Idle, Presenting) => {
            crate::overlay::present(app)?;
        }
        // Unlocked: tear the overlay down. Power policy is synchronized below.
        (Presenting, Idle) => {
            crate::overlay::dismiss(app);
        }
        // Failed auth -> Frozen: the native lock (SACLockScreenImmediate) is
        // already firing; KEEP our overlay up so the system lock covers it with
        // no desktop flash. Keep preventing system sleep: this is still the same
        // Veil lock session, and background workflows must remain alive.
        (Presenting, Frozen) => {}
        // Returned to Idle after macOS unlock: tear the overlay down.
        (Frozen, Idle) => crate::overlay::dismiss(app),
        _ => {}
    }

    sync_power(app);
    Ok(())
}

/// Whether the configured power assertion should be held in a given state.
fn should_prevent_sleep(state: State, enabled: bool) -> bool {
    enabled && state != State::Idle
}

/// Make the IOKit assertion match current settings and lock state. This is also
/// called after settings changes so toggling the option takes effect immediately.
pub(crate) fn sync_power(app: &AppHandle) {
    let managed = app.state::<Mutex<AppState>>();
    let mut guard = managed.lock().unwrap();
    let should_hold = should_prevent_sleep(guard.state, guard.settings.prevent_sleep);
    match (should_hold, guard.power_assertion_id) {
        (true, None) => match crate::power::acquire() {
            Ok(id) => {
                log::info!("prevent-sleep assertion acquired ({id})");
                guard.power_assertion_id = Some(id);
            }
            Err(e) => log::warn!("prevent-sleep assertion failed: {e}"),
        },
        (false, Some(id)) => {
            guard.power_assertion_id = None;
            crate::power::release(id);
            log::info!("prevent-sleep assertion released ({id})");
        }
        _ => {}
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

    #[test]
    fn prevents_sleep_for_the_entire_lock_session() {
        use State::*;
        assert!(!should_prevent_sleep(Idle, true));
        assert!(should_prevent_sleep(Presenting, true));
        assert!(should_prevent_sleep(Frozen, true));
        assert!(!should_prevent_sleep(Presenting, false));
        assert!(!should_prevent_sleep(Frozen, false));
    }
}
