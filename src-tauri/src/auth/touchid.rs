//! Touch ID via the LocalAuthentication framework.
//!
//! Policy is `DeviceOwnerAuthenticationWithBiometrics` (biometrics only) — NOT
//! `DeviceOwnerAuthentication`, which would surface the system *password* as a
//! fallback. Veil provides its own PIN fallback instead.
//!
//! `evaluatePolicy` is asynchronous (its reply block fires on a background
//! thread), so we bridge it to a synchronous result over a channel.

use super::AuthOutcome;

#[cfg(target_os = "macos")]
pub fn authenticate(reason: &str) -> AuthOutcome {
    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_foundation::{NSError, NSString};
    use objc2_local_authentication::{LAContext, LAPolicy};
    use std::sync::mpsc;

    let context = unsafe { LAContext::new() };
    let policy = LAPolicy::DeviceOwnerAuthenticationWithBiometrics;

    // Biometrics must be available and enrolled.
    if unsafe { context.canEvaluatePolicy_error(policy) }.is_err() {
        return AuthOutcome::Unavailable;
    }

    let (tx, rx) = mpsc::channel::<AuthOutcome>();
    let reason = NSString::from_str(reason);

    let reply = RcBlock::new(move |success: Bool, _error: *mut NSError| {
        let outcome = if success.as_bool() {
            AuthOutcome::Success
        } else {
            // A non-success result is either a user cancel/fallback tap or a
            // genuine failure. We can't distinguish cleanly without inspecting
            // the LAError code; treat both as Failure — the caller locks either
            // way, and a cancel from the biometric sheet means "I gave up".
            AuthOutcome::Failure
        };
        let _ = tx.send(outcome);
    });

    unsafe {
        context.evaluatePolicy_localizedReason_reply(policy, &reason, &reply);
    }

    // Block until the reply fires. A long timeout guards against the prompt
    // never returning (e.g. the sheet is dismissed by a system event).
    match rx.recv_timeout(std::time::Duration::from_secs(60)) {
        Ok(outcome) => outcome,
        Err(_) => AuthOutcome::Failure,
    }
}

#[cfg(not(target_os = "macos"))]
pub fn authenticate(_reason: &str) -> AuthOutcome {
    AuthOutcome::Unavailable
}
