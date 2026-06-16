//! Persisted settings, stored as JSON in the app config directory.
//!
//! Persisted structs use snake_case (serde default) so the TypeScript
//! `Settings` interface matches field-for-field.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub background_image_path: Option<String>,
    pub show_clock: bool,
    pub grace_timeout_ms: u64,
    pub prevent_sleep: bool,
    pub launch_at_login: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            background_image_path: None,
            show_clock: true,
            grace_timeout_ms: 0,
            prevent_sleep: true,
            launch_at_login: false,
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("no app config dir: {e}"))?;
    Ok(dir.join("settings.json"))
}

/// Load settings from disk, falling back to defaults when absent or unreadable.
pub fn load(app: &AppHandle) -> Settings {
    let path = match settings_path(app) {
        Ok(p) => p,
        Err(e) => {
            log::warn!("settings path unavailable, using defaults: {e}");
            return Settings::default();
        }
    };
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
            log::warn!("settings parse failed, using defaults: {e}");
            Settings::default()
        }),
        Err(_) => Settings::default(),
    }
}

/// Persist settings to disk (atomic write via temp file + rename).
pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
    }
    let json =
        serde_json::to_string_pretty(settings).map_err(|e| format!("serialize failed: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| format!("write failed: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("rename failed: {e}")
    })?;
    Ok(())
}
