//! Argon2id hashing and verification for the PIN and recovery code.
//!
//! Used by the auth/keychain commands wired in Phase 4/5; the `allow(dead_code)`
//! keeps the unused-warning quiet until then.
#![allow(dead_code)]

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;

/// Hash a secret (PIN or recovery code) with Argon2id, returning the encoded
/// PHC string (includes the salt).
pub fn hash_secret(secret: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("hash failed: {e}"))
}

/// Verify a secret against a previously stored Argon2id PHC string.
pub fn verify_secret(secret: &str, encoded: &str) -> bool {
    match PasswordHash::new(encoded) {
        Ok(parsed) => Argon2::default()
            .verify_password(secret.as_bytes(), &parsed)
            .is_ok(),
        Err(e) => {
            log::error!("stored hash is malformed: {e}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let hash = hash_secret("1234").unwrap();
        assert!(verify_secret("1234", &hash));
        assert!(!verify_secret("4321", &hash));
    }

    #[test]
    fn malformed_hash_does_not_verify() {
        assert!(!verify_secret("1234", "not-a-phc-string"));
    }
}
