//! macOS Keychain storage for the PIN hash.
//!
//! Stores the Argon2id PHC string (never the raw PIN) as a generic-password
//! item under the Veil service.

use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

const SERVICE: &str = "com.veil.app";
const PIN_ACCOUNT: &str = "pin-hash";

/// `errSecItemNotFound` — returned when a keychain item is absent.
const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;

pub fn store_pin_hash(hash: &str) -> Result<(), String> {
    set_generic_password(SERVICE, PIN_ACCOUNT, hash.as_bytes())
        .map_err(|e| format!("keychain store (pin): {e}"))
}

/// Read the PIN hash, returning `None` when no item exists yet.
pub fn read_pin_hash() -> Result<Option<String>, String> {
    match get_generic_password(SERVICE, PIN_ACCOUNT) {
        Ok(bytes) => {
            let s = String::from_utf8(bytes)
                .map_err(|e| format!("keychain pin item not utf-8: {e}"))?;
            Ok(Some(s))
        }
        Err(e) if e.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(None),
        Err(e) => Err(format!("keychain read (pin): {e}")),
    }
}

/// Remove the stored PIN (used when resetting credentials).
#[allow(dead_code)]
pub fn clear() {
    let _ = delete_generic_password(SERVICE, PIN_ACCOUNT);
}
