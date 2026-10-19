//! The tray popup: one frameless window anchored to the tray icon.
//!
//! Info mode (opened by the user) hides when it loses focus, unless the user
//! has pinned it. Dragged by its header, the popup reopens where it was left;
//! a double click on the header sends it back to the tray icon.
//! Reminder mode (opened by the timer) is pinned: it shows without taking
//! keyboard focus and stays up until the reminder is answered. If the user had
//! the popup pinned open, it stays open afterwards.

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
    /// Set from the pin button: blur no longer hides the popup, but the close
    /// button, Esc and the tray icon still do.
    user_pinned: AtomicBool,
    /// Whether the popup was already open when the current reminder came up.
    open_before_reminder: AtomicBool,
    /// Where the user dragged the popup to.
    dragged_position: Mutex<Option<PhysicalPosition<i32>>>,
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
    if let Some(pos) = saved_position(w, app) {
        return w.set_position(pos);
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

    let tray_located = app
        .state::<PopupState>()
        .tray_located
        .load(Ordering::Relaxed);
    let (x, y) = if tray_located {
        // TrayCenter sits above the icon, or below it when there's no room
        // above (macOS menu bar). TrayBottomCenter puts the popup's top edge at
        // the icon's top edge, i.e. inside the Windows taskbar and off screen.
        w.move_window(Position::TrayCenter)?;
        let pos = w.outer_position()?;
        (pos.x, pos.y)
    } else {
        let x = ax + aw - size.width - margin;
        let y = if cfg!(target_os = "macos") {
            ay + margin
        } else {
            ay + ah - size.height - margin
        };
        (x, y)
    };

    // Keep the whole popup inside the work area (clear of the taskbar / menu
    // bar and the screen edges, e.g. for a tray icon in the far corner).
    let x = x.clamp(ax, (ax + aw - size.width).max(ax));
    let y = y.clamp(ay, (ay + ah - size.height).max(ay));
    w.set_position(PhysicalPosition::new(x, y))
}

/// The popup's dragged-to position, if it's still on a connected screen.
fn saved_position<R: Runtime>(
    w: &WebviewWindow<R>,
    app: &AppHandle<R>,
) -> Option<PhysicalPosition<i32>> {
    let pos = (*app.state::<PopupState>().dragged_position.lock().unwrap())?;
    let on_screen = w.available_monitors().ok()?.iter().any(|m| {
        let area = m.work_area();
        let (x, y) = (area.position.x, area.position.y);
        (x..x + area.size.width as i32).contains(&pos.x)
            && (y..y + area.size.height as i32).contains(&pos.y)
    });
    on_screen.then_some(pos)
}

fn is_pinned<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.state::<PopupState>().pinned.load(Ordering::Relaxed)
}

pub fn show<R: Runtime>(app: &AppHandle<R>) {
    let Some(w) = window(app) else { return };
    if !w.is_visible().unwrap_or(false) {
        let _ = place(&w, app);
    }
    let _ = w.set_focusable(true);
    let _ = w.show();
    let _ = w.set_focus();
}

/// Opens the popup in reminder mode without stealing keyboard focus from
/// whatever the user is typing into.
pub fn show_reminder<R: Runtime>(app: &AppHandle<R>) {
    let Some(w) = window(app) else { return };
    let visible = w.is_visible().unwrap_or(false);
    let state = app.state::<PopupState>();
    if !state.pinned.swap(true, Ordering::Relaxed) {
        state.open_before_reminder.store(visible, Ordering::Relaxed);
    }
    if visible {
        return;
    }
    let _ = place(&w, app);
    let _ = w.set_focusable(false);
    show_inactive(&w);
}

/// Tao's `show` activates the window on Windows, so call ShowWindow directly.
#[cfg(windows)]
fn show_inactive<R: Runtime>(w: &WebviewWindow<R>) {
    use windows::Win32::UI::WindowsAndMessaging::{SW_SHOWNOACTIVATE, ShowWindow};
    if let Ok(hwnd) = w.hwnd() {
        // SAFETY: hwnd belongs to a live window owned by this process.
        let _ = unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
    }
}

/// A non-focusable window is ordered front without becoming key.
#[cfg(not(windows))]
fn show_inactive<R: Runtime>(w: &WebviewWindow<R>) {
    let _ = w.show();
}

/// Hides the popup unless a reminder is waiting for an answer.
pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if is_pinned(app) {
        return;
    }
    let Some(w) = window(app) else { return };
    let _ = w.hide();
    // `show_inactive` bypasses tao, so tao may think the window is already
    // hidden and skip hiding it.
    #[cfg(windows)]
    if let Ok(hwnd) = w.hwnd() {
        use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};
        // SAFETY: as in `show_inactive`.
        let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
    }
}

/// Leaves reminder mode once it has been answered, and hides the popup unless
/// the user had it pinned open.
pub fn release<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<PopupState>();
    state.pinned.store(false, Ordering::Relaxed);
    if let Some(w) = window(app) {
        let _ = w.set_focusable(true);
    }
    let keep_open = state.user_pinned.load(Ordering::Relaxed)
        && state.open_before_reminder.load(Ordering::Relaxed);
    if !keep_open {
        hide(app);
    }
}

pub fn toggle<R: Runtime>(app: &AppHandle<R>) {
    let Some(w) = window(app) else { return };
    if is_pinned(app) {
        // An open reminder can't be dismissed by clicking the tray; focus it.
        show(app);
        return;
    }
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

/// Dragging the popup moves keyboard focus off the webview while the popup
/// stays the active window; that isn't the user clicking away.
#[cfg(windows)]
fn still_active<R: Runtime>(w: &WebviewWindow<R>) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    // SAFETY: no arguments; returns a window handle or null.
    w.hwnd().is_ok_and(|hwnd| unsafe { GetForegroundWindow() } == hwnd)
}

#[cfg(not(windows))]
fn still_active<R: Runtime>(w: &WebviewWindow<R>) -> bool {
    w.is_focused().unwrap_or(false)
}

pub fn on_blur<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<PopupState>();
    if is_pinned(app) || state.user_pinned.load(Ordering::Relaxed) {
        return;
    }
    if window(app).is_some_and(|w| still_active(&w)) {
        return;
    }
    *app.state::<PopupState>().last_blur_hide.lock().unwrap() = Some(Instant::now());
    hide(app);
}

/// Remembers where the user dragged the popup. `place` only moves it while
/// hidden, so moves of the visible popup are the user's.
pub fn on_moved<R: Runtime>(app: &AppHandle<R>, pos: PhysicalPosition<i32>) {
    let visible = window(app).is_some_and(|w| w.is_visible().unwrap_or(false));
    if visible {
        *app.state::<PopupState>().dragged_position.lock().unwrap() = Some(pos);
    }
}

#[tauri::command]
pub fn hide_popup(app: AppHandle) {
    hide(&app);
}

#[tauri::command]
pub fn set_popup_pinned(app: AppHandle, pinned: bool) {
    app.state::<PopupState>()
        .user_pinned
        .store(pinned, Ordering::Relaxed);
}

/// Forgets the dragged position and moves the popup back to the tray icon.
#[tauri::command]
pub fn reset_popup_position(app: AppHandle) {
    *app.state::<PopupState>().dragged_position.lock().unwrap() = None;
    if let Some(w) = window(&app) {
        let _ = place(&w, &app);
    }
}
