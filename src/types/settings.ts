/**
 * Mirror of the Rust persisted `Settings` struct (src-tauri/src/settings.rs).
 * Persisted data uses snake_case to match Rust's default serde naming.
 */
export interface Settings {
  background_image_path: string | null
  show_clock: boolean
  grace_timeout_ms: number
  prevent_sleep: boolean
  launch_at_login: boolean
}

export const defaultSettings: Settings = {
  background_image_path: null,
  show_clock: true,
  grace_timeout_ms: 0,
  prevent_sleep: true,
  launch_at_login: false,
}
