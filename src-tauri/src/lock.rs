//! Native macOS lock fallback.
//!
//! Uses the Cmd+Ctrl+Q keystroke trick via `osascript`, which locks the screen
//! regardless of the "require password after sleep" setting (chosen over
//! `pmset displaysleepnow`, which only locks when that setting is enabled).
//!
//! Phase 1 stub; Phase 4 invokes osascript.

pub fn lock_screen() -> Result<(), String> {
    // Phase 4:
    //   osascript -e 'tell application "System Events" to keystroke "q"
    //                 using {command down, control down}'
    Err("lock not implemented yet".into())
}
