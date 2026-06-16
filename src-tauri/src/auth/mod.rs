//! Authentication: Touch ID, then PIN. There is no recovery code — if the PIN
//! is forgotten, the macOS lock screen is the fallback (fail auth → macOS lock →
//! log into the Mac → change the PIN in Settings).

pub mod pin;
mod touchid;

pub use touchid::authenticate;

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthOutcome {
    Success,
    Failure,
    Unavailable,
}

// Cache for "is a PIN configured" so we don't hit the Keychain (which prompts
// for access on unsigned/dev builds) on every launch and every lock. The PIN's
// *existence* isn't sensitive — only the hash is, and that's still read from the
// Keychain on an actual unlock. 0 = unknown, 1 = no, 2 = yes.
static PIN_CONFIGURED: AtomicU8 = AtomicU8::new(0);

/// Whether a PIN has been configured (a Keychain pin-hash item exists). Cached
/// after the first Keychain read.
pub fn is_pin_configured() -> bool {
    match PIN_CONFIGURED.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let configured = matches!(crate::keychain::read_pin_hash(), Ok(Some(_)));
            PIN_CONFIGURED.store(if configured { 2 } else { 1 }, Ordering::Relaxed);
            configured
        }
    }
}

/// Store a new PIN (hashed). Used by first-run setup and PIN change.
pub fn set_pin(pin: &str) -> Result<(), String> {
    let hash = pin::hash_secret(pin)?;
    crate::keychain::store_pin_hash(&hash)?;
    PIN_CONFIGURED.store(2, Ordering::Relaxed);
    // New PIN → invalidate the cached hash so the next verify reads the fresh one.
    *PIN_HASH_CACHE.lock().unwrap() = Some(hash);
    Ok(())
}

// In-memory cache of the PIN hash, populated on first read. Avoids re-prompting
// for Keychain access on every unlock attempt within a session (the unsigned
// dev binary re-prompts otherwise). It's the argon2 hash, never the raw PIN, and
// the process already does the verification — so this doesn't widen exposure.
static PIN_HASH_CACHE: Mutex<Option<String>> = Mutex::new(None);

/// Verify a PIN against the stored hash (cached in memory after first read).
pub fn verify_pin(pin: &str) -> bool {
    let hash = {
        let mut cache = PIN_HASH_CACHE.lock().unwrap();
        if cache.is_none() {
            *cache = crate::keychain::read_pin_hash().ok().flatten();
        }
        cache.clone()
    };
    match hash {
        Some(h) => pin::verify_secret(pin, &h),
        None => false,
    }
}
