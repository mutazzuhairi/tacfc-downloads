//! TAC FC desktop (direct download from tacfc.com/download, not a store build).
//!
//! One window. It opens the bundled www/index.html, which checks the connection and then goes to
//! https://tacfc.com/?source=direct (or shows the offline page with a retry). The user agent carries
//! TACFCDirect/1.0, so the site keeps purchases visible (apps/web/lib/nativeApp.ts, the direct flavour).
//! The remote site gets no Tauri IPC (capabilities cover the local page only).
//! On start the updater checks latest.json (tauri.conf.json, plugins.updater) and offers a restart.

use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_updater::UpdaterExt;

/// The suffix the site looks for (DIRECT_UA_TOKEN in apps/web/lib/nativeApp.ts).
const DIRECT_UA: &str = "TACFCDirect/1.0";

/// A normal browser user agent for this system plus the direct token (the web views give no way to append).
fn user_agent() -> String {
    let base = if cfg!(target_os = "windows") {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36 Edg/140.0.0.0"
    } else if cfg!(target_os = "macos") {
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15"
    } else {
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36"
    };
    format!("{base} {DIRECT_UA}")
}

/// Checks for a new version once at start. Quiet when there is none or the check fails (offline).
async fn check_update(app: tauri::AppHandle) {
    let Ok(updater) = app.updater() else { return };
    let Ok(Some(update)) = updater.check().await else { return };
    let version = update.version.clone();
    if update.download_and_install(|_, _| {}, || {}).await.is_err() {
        return;
    }
    let handle = app.clone();
    app.dialog()
        .message(format!("TAC FC {version} is installed. Restart now to use it?"))
        .title("TAC FC update")
        .buttons(MessageDialogButtons::OkCancelCustom("Restart now".into(), "Later".into()))
        .show(move |restart| {
            if restart {
                handle.restart();
            }
        });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("TAC FC")
                .inner_size(1280.0, 820.0)
                .min_inner_size(380.0, 600.0)
                .center()
                .user_agent(&user_agent())
                .build()?;
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(check_update(handle));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("TAC FC could not start");
}
