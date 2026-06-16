//! Authentication orchestration: Touch ID first, then PIN, then recovery code.

pub mod pin;
mod touchid;

pub use touchid::authenticate;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthOutcome {
    Success,
    Failure,
    Unavailable,
}

/// Verify a PIN against the stored hash. Phase 4 uses a hardcoded test hash;
/// Phase 5 repoints `stored_pin_hash()` at the Keychain.
pub fn verify_pin(pin: &str) -> bool {
    match stored_pin_hash() {
        Some(hash) => pin::verify_secret(pin, &hash),
        None => false,
    }
}

/// Verify a recovery code against the stored hash.
pub fn verify_recovery(code: &str) -> bool {
    match stored_recovery_hash() {
        Some(hash) => pin::verify_secret(&normalize_recovery(code), &hash),
        None => false,
    }
}

/// Normalize a recovery code for comparison: uppercase, strip spaces/dashes.
pub fn normalize_recovery(code: &str) -> String {
    code.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase()
}

// ── Temporary credential store (Phase 4) ─────────────────────────────────────
// Phase 5 replaces these with Keychain reads. The test PIN is "1234" and the
// test recovery code is "ABCD-EFGH-JKLM-NPQR" (normalized "ABCDEFGHJKLMNPQR").
// These hashes were generated once with auth::pin::hash_secret.

fn stored_pin_hash() -> Option<String> {
    Some(TEST_PIN_HASH.to_string())
}

fn stored_recovery_hash() -> Option<String> {
    Some(TEST_RECOVERY_HASH.to_string())
}

const TEST_PIN_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$8V6Im3pLZ7lE6KP3SSRPsw$2JFi7JYTbfIARpwwduFn60AM3XQmuPrLT9+BfKwI0pI";
const TEST_RECOVERY_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$04QeTK+gHeRQF/hleF+zsQ$VoLGLsvlFMpLHaIegYLY/iFUiGZA2dkNszx+qAOHsF8";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_normalization() {
        assert_eq!(normalize_recovery("abcd-efgh"), "ABCDEFGH");
        assert_eq!(normalize_recovery("ABCD EFGH"), "ABCDEFGH");
        assert_eq!(normalize_recovery("a1-b2"), "A1B2");
    }
}
