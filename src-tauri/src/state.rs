//! Core state machine. Every state transition flows through `transition()`,
//! which validates the edge, runs side effects, and emits `state-changed`.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::settings::Settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Idle,
    Armed,
    Presenting,
    Frozen,
}

/// App-wide state, managed as `Mutex<AppState>`.
///
/// A `std::sync::Mutex` (not tokio's) is correct here: transitions are
/// synchronous and short, and no `.await` is ever held across the lock.
pub struct AppState {
    pub state: State,
    /// IOKit power-assertion id held while Armed (read once acquire/release
    /// land in Phase 8).
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
/// Idle ──arm──► Armed ──focus loss──► Presenting
///   ▲             │ ▲ auth ok            │
///   │ disarm      │ └────────────────────┘
///   │             │ auth fail/dismiss ──► Frozen ──resume──► Armed
/// ```
/// `* -> Idle` is always legal (disarm / quit).
pub fn is_legal(from: State, to: State) -> bool {
    use State::*;
    if to == Idle {
        return true;
    }
    matches!(
        (from, to),
        (Idle, Armed)
            | (Armed, Presenting)
            | (Presenting, Armed)
            | (Presenting, Frozen)
            | (Frozen, Armed)
    )
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

/// Side effects attached to specific edges. Filled in across later phases
/// (overlay present/dismiss, power assertion, native lock).
fn run_side_effects(app: &AppHandle, from: State, to: State) {
    use State::*;
    match (from, to) {
        (Armed, Presenting) => crate::overlay::present(app),
        (Presenting, Armed) | (Presenting, Frozen) => crate::overlay::dismiss(app),
        _ => {}
    }
    // Power-assertion acquire/release (Phase 8) and native lock on -> Frozen
    // (Phase 4) hook in here.
    let _ = (from, to);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_edges() {
        use State::*;
        assert!(is_legal(Idle, Armed));
        assert!(is_legal(Armed, Presenting));
        assert!(is_legal(Presenting, Armed));
        assert!(is_legal(Presenting, Frozen));
        assert!(is_legal(Frozen, Armed));
        // disarm from anywhere
        assert!(is_legal(Armed, Idle));
        assert!(is_legal(Presenting, Idle));
        assert!(is_legal(Frozen, Idle));
    }

    #[test]
    fn illegal_edges() {
        use State::*;
        assert!(!is_legal(Idle, Presenting));
        assert!(!is_legal(Idle, Frozen));
        assert!(!is_legal(Armed, Frozen));
        assert!(!is_legal(Frozen, Presenting));
    }
}
