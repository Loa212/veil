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
    /// IOKit power-assertion id held while the overlay is up (read once
    /// acquire/release land in Phase 8).
    #[allow(dead_code)]
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
        // Lock now: raise the overlay.
        (Idle, Presenting) => crate::overlay::present(app),
        // Leaving Presenting (unlocked to Idle, or failed to Frozen): tear down.
        (Presenting, Idle) | (Presenting, Frozen) => crate::overlay::dismiss(app),
        _ => {}
    }
    // Power-assertion acquire/release hooks in here (Phase 8).
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
