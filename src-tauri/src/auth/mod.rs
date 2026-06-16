//! Authentication: Touch ID, then PIN. There is no recovery code — if the PIN
//! is forgotten, the macOS lock screen is the fallback (fail auth → macOS lock →
//! log into the Mac → change the PIN in Settings).

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

/// Verify a PIN against the Keychain-stored hash.
pub fn verify_pin(pin: &str) -> bool {
    match crate::keychain::read_pin_hash() {
        Ok(Some(hash)) => pin::verify_secret(pin, &hash),
        _ => false,
    }
}
