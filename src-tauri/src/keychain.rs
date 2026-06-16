//! macOS Keychain storage for the PIN hash and recovery-code hash.
//!
//! Stores the Argon2id PHC strings (never the raw secrets) as generic-password
//! items under the Veil service.

use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

const SERVICE: &str = "com.veil.app";
const PIN_ACCOUNT: &str = "pin-hash";
const RECOVERY_ACCOUNT: &str = "recovery-hash";

fn store(account: &str, hash: &str) -> Result<(), String> {
    set_generic_password(SERVICE, account, hash.as_bytes())
        .map_err(|e| format!("keychain store ({account}): {e}"))
}

/// Read a hash item, returning `None` when the item doesn't exist yet.
fn read(account: &str) -> Result<Option<String>, String> {
    match get_generic_password(SERVICE, account) {
        Ok(bytes) => {
            let s = String::from_utf8(bytes)
                .map_err(|e| format!("keychain item ({account}) not utf-8: {e}"))?;
            Ok(Some(s))
        }
        Err(e) if e.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(None),
        Err(e) => Err(format!("keychain read ({account}): {e}")),
    }
}

/// `errSecItemNotFound` — returned when a keychain item is absent.
const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;

pub fn store_pin_hash(hash: &str) -> Result<(), String> {
    store(PIN_ACCOUNT, hash)
}

pub fn read_pin_hash() -> Result<Option<String>, String> {
    read(PIN_ACCOUNT)
}

pub fn store_recovery_hash(hash: &str) -> Result<(), String> {
    store(RECOVERY_ACCOUNT, hash)
}

pub fn read_recovery_hash() -> Result<Option<String>, String> {
    read(RECOVERY_ACCOUNT)
}

/// Remove both items (used when resetting credentials).
#[allow(dead_code)]
pub fn clear() {
    let _ = delete_generic_password(SERVICE, PIN_ACCOUNT);
    let _ = delete_generic_password(SERVICE, RECOVERY_ACCOUNT);
}
