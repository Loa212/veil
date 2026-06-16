//! Auto-update via tauri-plugin-updater. The update "server" is just GitHub
//! Releases: the app fetches `latest.json` from the configured endpoint, and if
//! a newer version exists, prompts to download + install it (artifacts are
//! verified against the minisign pubkey in tauri.conf.json).

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_updater::UpdaterExt;

/// Check for an update. `silent` suppresses the "you're up to date" / error
/// notices (used by the on-launch auto-check); the manual tray check passes
/// `false` so the user always gets feedback.
pub fn check(app: &AppHandle, silent: bool) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match app.updater() {
            Ok(updater) => match updater.check().await {
                Ok(Some(update)) => prompt_and_install(&app, update).await,
                Ok(None) => {
                    if !silent {
                        info(
                            &app,
                            "You're up to date",
                            "Veil is running the latest version.",
                        );
                    }
                }
                Err(e) => {
                    log::warn!("update check failed: {e}");
                    if !silent {
                        info(&app, "Update check failed", &format!("{e}"));
                    }
                }
            },
            Err(e) => log::warn!("updater unavailable: {e}"),
        }
    });
}

async fn prompt_and_install(app: &AppHandle, update: tauri_plugin_updater::Update) {
    let version = update.version.clone();
    let install = app
        .dialog()
        .message(format!(
            "Veil {version} is available. Download and install it now?"
        ))
        .title("Update available")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Install".into(),
            "Later".into(),
        ))
        .blocking_show();

    if !install {
        return;
    }

    log::info!("installing update {version}");
    match update.download_and_install(|_, _| {}, || {}).await {
        Ok(()) => {
            app.dialog()
                .message("Update installed. Veil will now restart.")
                .title("Update complete")
                .blocking_show();
            app.restart();
        }
        Err(e) => {
            log::error!("update install failed: {e}");
            info(app, "Update failed", &format!("{e}"));
        }
    }
}

fn info(app: &AppHandle, title: &str, body: &str) {
    app.dialog().message(body).title(title).blocking_show();
}
