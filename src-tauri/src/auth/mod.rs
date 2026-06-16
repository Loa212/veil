//! Authentication orchestration: Touch ID first, then PIN, then recovery code.

pub mod pin;
mod touchid;

pub use touchid::authenticate;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
// Success/Failure/Cancelled are produced once Touch ID lands in Phase 4.
#[allow(dead_code)]
pub enum AuthOutcome {
    Success,
    Failure,
    Cancelled,
    Unavailable,
}
