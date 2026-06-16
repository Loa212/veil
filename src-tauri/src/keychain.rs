//! macOS Keychain storage for the PIN hash and recovery-code hash.
//!
//! Phase 1 stub. Phase 5 implements generic-password items via
//! `security-framework`, scoped to the Veil service.

#![allow(dead_code)]

const SERVICE: &str = "com.veil.app";
const PIN_ACCOUNT: &str = "pin-hash";
const RECOVERY_ACCOUNT: &str = "recovery-hash";

pub fn store_pin_hash(_hash: &str) -> Result<(), String> {
    // Phase 5.
    Err("keychain not implemented yet".into())
}

pub fn read_pin_hash() -> Result<Option<String>, String> {
    // Phase 5.
    Ok(None)
}

pub fn store_recovery_hash(_hash: &str) -> Result<(), String> {
    // Phase 5.
    Err("keychain not implemented yet".into())
}

pub fn read_recovery_hash() -> Result<Option<String>, String> {
    // Phase 5.
    Ok(None)
}

// Silence unused-const warnings until Phase 5 wires these in.
const _: (&str, &str, &str) = (SERVICE, PIN_ACCOUNT, RECOVERY_ACCOUNT);
