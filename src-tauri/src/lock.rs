//! Native macOS lock fallback.
//!
//! Sends the Cmd+Ctrl+Q lock-screen keystroke via `osascript` / System Events.
//! Unlike `pmset displaysleepnow`, this locks the session regardless of the
//! "require password after sleep" setting.
//!
//! Note: sending synthetic keystrokes requires Accessibility permission for
//! Veil (System Settings → Privacy & Security → Accessibility). The first
//! attempt will prompt for it.

use std::process::Command;

const LOCK_SCRIPT: &str =
    r#"tell application "System Events" to keystroke "q" using {command down, control down}"#;

pub fn lock_screen() -> Result<(), String> {
    let status = Command::new("osascript")
        .args(["-e", LOCK_SCRIPT])
        .status()
        .map_err(|e| format!("failed to run osascript: {e}"))?;

    if status.success() {
        log::info!("native lock triggered");
        Ok(())
    } else {
        Err(format!("osascript exited with {status}"))
    }
}
