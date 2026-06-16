//! IOKit power assertion to prevent system idle sleep while Armed.
//!
//! Phase 1 stub. Phase 8 implements `IOPMAssertionCreateWithName` /
//! `IOPMAssertionRelease` via FFI to the IOKit framework.

#![allow(dead_code)]

/// Acquire an assertion preventing idle system sleep. Returns its id.
pub fn acquire() -> Result<u32, String> {
    // Phase 8.
    Err("power assertion not implemented yet".into())
}

/// Release a previously-acquired assertion.
pub fn release(_id: u32) {
    // Phase 8.
}
