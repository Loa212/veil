//! Recovery-code generation. Produces a human-readable code shown once at
//! setup; only its Argon2 hash is persisted.
//!
//! Wired into the first-run/recovery commands in Phase 5.
#![allow(dead_code)]

use rand::Rng;

/// Unambiguous alphabet (no 0/O, 1/I/L).
const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// Generate a recovery code formatted as 4 groups of 4, e.g. `A3F9-K2M7-...`.
pub fn generate() -> String {
    let mut rng = rand::thread_rng();
    let mut groups = Vec::with_capacity(4);
    for _ in 0..4 {
        let group: String = (0..4)
            .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
            .collect();
        groups.push(group);
    }
    groups.join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_is_four_groups_of_four() {
        let code = generate();
        let groups: Vec<&str> = code.split('-').collect();
        assert_eq!(groups.len(), 4);
        assert!(groups.iter().all(|g| g.len() == 4));
    }
}
