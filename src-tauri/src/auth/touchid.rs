//! Touch ID via the LocalAuthentication framework.
//!
//! Policy is `DeviceOwnerAuthenticationWithBiometrics` (biometrics only) — NOT
//! `DeviceOwnerAuthentication`, which would surface the system *password* as a
//! fallback. Veil provides its own PIN fallback instead.
//!
//! `evaluatePolicy` must be started from a thread with an active run loop, and
//! its reply block is delivered there. A Tauri command worker thread has no run
//! loop, so we dispatch the evaluation onto the main thread (which has the app's
//! run loop) and block the command thread on a channel until the reply fires.

use super::AuthOutcome;
use tauri::AppHandle;

#[cfg(target_os = "macos")]
pub fn authenticate(app: &AppHandle, reason: &str) -> AuthOutcome {
    use block2::RcBlock;
    use objc2::rc::Retained;
    use objc2::runtime::Bool;
    use objc2_foundation::{NSError, NSString};
    use objc2_local_authentication::{LAContext, LAPolicy};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel::<AuthOutcome>();
    let reason = reason.to_string();
    let tx_dispatch = tx.clone();

    // Start the evaluation on the main thread so its reply block is delivered by
    // the app's run loop.
    let dispatched = app.run_on_main_thread(move || {
        let context: Retained<LAContext> = unsafe { LAContext::new() };
        let policy = LAPolicy::DeviceOwnerAuthenticationWithBiometrics;

        if unsafe { context.canEvaluatePolicy_error(policy) }.is_err() {
            let _ = tx_dispatch.send(AuthOutcome::Unavailable);
            return;
        }

        let ns_reason = NSString::from_str(&reason);
        let tx_reply = tx_dispatch.clone();

        // Keep the LAContext alive until the reply fires by moving it into the
        // block alongside the sender.
        let ctx_keepalive = context.clone();
        let reply = RcBlock::new(move |success: Bool, _error: *mut NSError| {
            let _ = &ctx_keepalive; // hold the context for the call's duration
            let outcome = if success.as_bool() {
                AuthOutcome::Success
            } else {
                AuthOutcome::Failure
            };
            let _ = tx_reply.send(outcome);
        });

        unsafe {
            context.evaluatePolicy_localizedReason_reply(policy, &ns_reason, &reply);
        }
    });

    if dispatched.is_err() {
        return AuthOutcome::Unavailable;
    }

    match rx.recv_timeout(std::time::Duration::from_secs(60)) {
        Ok(outcome) => outcome,
        Err(_) => AuthOutcome::Failure,
    }
}

#[cfg(not(target_os = "macos"))]
pub fn authenticate(_app: &AppHandle, _reason: &str) -> AuthOutcome {
    AuthOutcome::Unavailable
}
