//! The tray popup: one frameless window anchored to the tray icon.
//!
//! Info mode (opened by the user) hides when it loses focus.
//! Reminder mode (milestone 3) will set `pinned` so it stays up until answered.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, Runtime, WebviewWindow};
use tauri_plugin_positioner::{Position, WindowExt};

pub const LABEL: &str = "main";

/// Must match the window size in tauri.conf.json.
const SIZE: LogicalSize<f64> = LogicalSize {
    width: 340.0,
    height: 480.0,
};
/// Gap between the popup and the screen edge when the tray isn't located yet.
const EDGE_MARGIN: f64 = 12.0;

/// Clicking the tray icon while the popup is open first blurs the popup (which
/// hides it) and then delivers the click (which would show it again). Ignore
/// tray clicks that land this soon after a blur-hide.
const REOPEN_GUARD: Duration = Duration::from_millis(300);

#[derive(Default)]
pub struct PopupState {
    last_blur_hide: Mutex<Option<Instant>>,
    /// Set once the tray has reported its on-screen position to the positioner.
    tray_located: AtomicBool,
    /// When true the popup ignores blur (reminder mode).
    pinned: AtomicBool,
}

fn window<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window(LABEL)
}

pub fn mark_tray_located<R: Runtime>(app: &AppHandle<R>) {
    app.state::<PopupState>()
        .tray_located
        .store(true, Ordering::Relaxed);
}

/// Anchor the popup to the tray icon once its position is known; until then
/// (e.g. opened from a second launch) use the top-right (macOS) or
/// bottom-right (Windows) corner of the screen's work area.
fn place<R: Runtime>(w: &WebviewWindow<R>, app: &AppHandle<R>) -> tauri::Result<()> {
    if app
        .state::<PopupState>()
        .tray_located
        .load(Ordering::Relaxed)
    {
        // macOS: menu bar is at the top, popup hangs below the icon.
        // Windows: taskbar is usually at the bottom, popup sits above the icon.
        let position = if cfg!(target_os = "macos") {
            Position::TrayCenter
        } else {
            Position::TrayBottomCenter
        };
        return w.move_window(position);
    }

    let Some(monitor) = w.current_monitor()?.or(w.primary_monitor()?) else {
        return Ok(());
    };
    // Computed from the configured size: a window that has never been shown
    // can report 0x0 on some platforms.
    let scale = monitor.scale_factor();
    let size = SIZE.to_physical::<i32>(scale);
    let margin = (EDGE_MARGIN * scale) as i32;
    let area = monitor.work_area();
    let (ax, ay) = (area.position.x, area.position.y);
    let (aw, ah) = (area.size.width as i32, area.size.height as i32);

    let x = ax + aw - size.width - margin;
    let y = if cfg!(target_os = "macos") {
        ay + margin
    } else {
        ay + ah - size.height - margin
    };
    w.set_position(PhysicalPosition::new(x, y))
}

pub fn show<R: Runtime>(app: &AppHandle<R>) {
    let Some(w) = window(app) else { return };
    let _ = place(&w, app);
    let _ = w.show();
    let _ = w.set_focus();
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = window(app) {
        let _ = w.hide();
    }
}

pub fn toggle<R: Runtime>(app: &AppHandle<R>) {
    let Some(w) = window(app) else { return };
    if w.is_visible().unwrap_or(false) {
        hide(app);
        return;
    }
    let state = app.state::<PopupState>();
    let recently_hidden = state
        .last_blur_hide
        .lock()
        .unwrap()
        .is_some_and(|t| t.elapsed() < REOPEN_GUARD);
    if !recently_hidden {
        show(app);
    }
}

pub fn on_blur<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<PopupState>();
    if state.pinned.load(Ordering::Relaxed) {
        return;
    }
    *state.last_blur_hide.lock().unwrap() = Some(Instant::now());
    hide(app);
}

#[tauri::command]
pub fn hide_popup(app: AppHandle) {
    hide(&app);
}
