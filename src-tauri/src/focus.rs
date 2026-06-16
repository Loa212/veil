//! Focus-loss detection.
//!
//! Observes `NSApplicationDidResignActiveNotification` — fired when the user
//! switches away from Veil (Cmd+Tab, clicking another app, Mission Control).
//! On the `Armed -> Presenting` edge *only* we present the overlay; the
//! notification is ignored in every other state. That guard is what stops the
//! overlay's own focus churn (spawning key/main windows) from re-triggering
//! presentation.

use tauri::AppHandle;

#[cfg(target_os = "macos")]
pub fn install(app: &AppHandle) {
    use block2::RcBlock;
    use objc2_app_kit::NSApplicationDidResignActiveNotification;
    use objc2_foundation::NSNotificationCenter;
    use std::ptr::NonNull;

    let app = app.clone();

    // The block fires on the main thread (the notification is posted there and
    // we deliver on the posting thread by passing `queue: None`).
    let block = RcBlock::new(move |_note: NonNull<objc2_foundation::NSNotification>| {
        on_resign_active(&app);
    });

    // SAFETY: standard NSNotificationCenter registration. The observer token is
    // intentionally leaked so the observation lasts the whole process lifetime.
    unsafe {
        let center = NSNotificationCenter::defaultCenter();
        let token = center.addObserverForName_object_queue_usingBlock(
            Some(NSApplicationDidResignActiveNotification),
            None,
            None,
            &block,
        );
        std::mem::forget(token);
    }

    log::debug!("focus watcher installed");
}

#[cfg(target_os = "macos")]
fn on_resign_active(app: &AppHandle) {
    use crate::state::{self, State};

    if state::current(app) == State::Armed {
        log::debug!("resign-active while Armed → presenting overlay");
        state::transition(app, State::Presenting);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn install(_app: &AppHandle) {}
