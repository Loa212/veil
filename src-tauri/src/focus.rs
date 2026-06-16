//! Focus-loss detection. Observes `NSApplicationDidResignActiveNotification`
//! and arms the overlay on the `Armed -> Presenting` edge only — the Presenting
//! guard prevents the overlay's own focus churn from re-triggering.
//!
//! Phase 1 stub. Phase 3 registers the NSNotificationCenter observer.

use tauri::AppHandle;

pub fn install(_app: &AppHandle) {
    // Phase 3: addObserverForName:NSApplicationDidResignActiveNotification.
}
