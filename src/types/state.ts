/**
 * Mirror of the Rust `State` enum (src-tauri/src/state.rs).
 * Serialized lowercase via `#[serde(rename_all = "lowercase")]`.
 */
export type AppState = 'idle' | 'presenting' | 'frozen'

/** Auth stage shown inside the overlay's AuthPrompt. */
export type AuthStage = 'idle' | 'touchid' | 'pin' | 'recovery'

/** Outcome of an authentication attempt (mirror of Rust `AuthOutcome`). */
export type AuthOutcome = 'success' | 'failure' | 'cancelled' | 'unavailable'
