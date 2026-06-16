//! Observe the macOS screen-unlock event so Veil can auto-clear `Frozen`.
//!
//! After a failed-auth lock, the Mac shows the native login window. Once the
//! user logs back in they've already re-authenticated to macOS, so staying in
//! `Frozen` (requiring a separate "Resume") is redundant friction. We listen for
//! the distributed `com.apple.screenIsUnlocked` notification and transition
//! `Frozen -> Idle` automatically.

use tauri::AppHandle;

#[cfg(target_os = "macos")]
pub fn install(app: &AppHandle) {
    use block2::RcBlock;
    use objc2_foundation::{NSDistributedNotificationCenter, NSNotification, NSString};
    use std::ptr::NonNull;

    let app = app.clone();

    let block = RcBlock::new(move |_note: NonNull<NSNotification>| {
        on_screen_unlocked(&app);
    });

    // SAFETY: standard distributed-notification registration; the observer token
    // is leaked so the observation lasts the whole process lifetime.
    unsafe {
        let center = NSDistributedNotificationCenter::defaultCenter();
        let name = NSString::from_str("com.apple.screenIsUnlocked");
        let token =
            center.addObserverForName_object_queue_usingBlock(Some(&name), None, None, &block);
        std::mem::forget(token);
    }

    log::debug!("screen-unlock watcher installed");
}

#[cfg(target_os = "macos")]
fn on_screen_unlocked(app: &AppHandle) {
    use crate::state::{self, State};

    if state::current(app) == State::Frozen {
        log::info!("screen unlocked → clearing Frozen");
        state::transition(app, State::Idle);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn install(_app: &AppHandle) {}
