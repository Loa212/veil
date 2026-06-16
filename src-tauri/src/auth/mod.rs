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

/// Whether a PIN has been configured (a Keychain pin-hash item exists).
pub fn is_pin_configured() -> bool {
    matches!(crate::keychain::read_pin_hash(), Ok(Some(_)))
}

/// Store a new PIN (hashed). Used by first-run setup and PIN change.
pub fn set_pin(pin: &str) -> Result<(), String> {
    let hash = pin::hash_secret(pin)?;
    crate::keychain::store_pin_hash(&hash)
}

/// Generate, store (hashed), and return a fresh recovery code (shown once).
pub fn set_new_recovery() -> Result<String, String> {
    let code = crate::recovery::generate();
    let hash = pin::hash_secret(&normalize_recovery(&code))?;
    crate::keychain::store_recovery_hash(&hash)?;
    Ok(code)
}

/// Verify a PIN against the Keychain-stored hash.
pub fn verify_pin(pin: &str) -> bool {
    match crate::keychain::read_pin_hash() {
        Ok(Some(hash)) => pin::verify_secret(pin, &hash),
        _ => false,
    }
}

/// Verify a recovery code against the Keychain-stored hash.
pub fn verify_recovery(code: &str) -> bool {
    match crate::keychain::read_recovery_hash() {
        Ok(Some(hash)) => pin::verify_secret(&normalize_recovery(code), &hash),
        _ => false,
    }
}

/// Normalize a recovery code for comparison: uppercase, strip spaces/dashes.
pub fn normalize_recovery(code: &str) -> String {
    code.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase()
}

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
