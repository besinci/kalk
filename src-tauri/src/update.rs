//! Checks GitHub Releases for a newer version and installs it when the user
//! clicks Update in the popup. Release builds only: a dev build would offer
//! to replace itself with the released app.

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_updater::{Update, UpdaterExt};

pub const UPDATE_EVENT: &str = "update";
const FIRST_CHECK: Duration = Duration::from_secs(30);
const CHECK_EVERY: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Default)]
pub struct Updates {
    pending: Mutex<Option<Update>>,
}

#[derive(Clone, Serialize)]
pub struct UpdateInfo {
    version: String,
}

pub fn start<R: Runtime>(app: &AppHandle<R>) {
    app.manage(Updates::default());
    if cfg!(debug_assertions) {
        return;
    }
    let app = app.clone();
    thread::spawn(move || {
        thread::sleep(FIRST_CHECK);
        loop {
            tauri::async_runtime::block_on(check(&app));
            thread::sleep(CHECK_EVERY);
        }
    });
}

async fn check<R: Runtime>(app: &AppHandle<R>) {
    let update = match app.updater() {
        Ok(updater) => updater.check().await,
        Err(e) => Err(e),
    };
    match update {
        Ok(Some(update)) => {
            let info = UpdateInfo {
                version: update.version.clone(),
            };
            *app.state::<Updates>().pending.lock().unwrap() = Some(update);
            let _ = app.emit(UPDATE_EVENT, &info);
        }
        Ok(None) => {}
        // Offline, rate-limited, … : try again next time.
        Err(e) => eprintln!("kalk: update check failed: {e}"),
    }
}

#[tauri::command]
pub fn get_update(app: AppHandle) -> Option<UpdateInfo> {
    app.state::<Updates>()
        .pending
        .lock()
        .unwrap()
        .as_ref()
        .map(|u| UpdateInfo {
            version: u.version.clone(),
        })
}

/// Downloads and installs the pending update, then restarts into it.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = app.state::<Updates>().pending.lock().unwrap().take();
    let Some(update) = update else {
        return Err("no update available".into());
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;
    app.restart();
}
