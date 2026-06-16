//! Touch ID via the LocalAuthentication framework.
//!
//! Phase 1 stub: reports `Unavailable`. Phase 4 implements `LAContext` +
//! `evaluatePolicy:.deviceOwnerAuthenticationWithBiometrics` with a block2 reply.

use super::AuthOutcome;

pub fn authenticate(_reason: &str) -> AuthOutcome {
    // Phase 4.
    AuthOutcome::Unavailable
}
