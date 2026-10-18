//! Runs the timer in the background and wires it to the popup, the tray, the
//! settings and the log. The popup gets a `status` event on every tick.

use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use chrono::{Local, NaiveDateTime, NaiveTime};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::db::Db;
use crate::settings::{self, Settings};
use crate::stats::{self, Stats};
use crate::timer::{Engine, Event, Outcome, Phase};
use crate::{i18n, idle, popup, sound, tray};

pub const STATUS_EVENT: &str = "status";
const DEV: bool = cfg!(debug_assertions);

pub struct Breaks {
    engine: Mutex<Engine>,
    settings: Mutex<Settings>,
    settings_path: Option<PathBuf>,
    db: Option<Mutex<Db>>,
    tooltip: Mutex<String>,
}

#[derive(Clone, Serialize)]
pub struct Status {
    phase: Phase,
    secs_left: u64,
    /// Local time, ISO 8601.
    paused_until: Option<String>,
    done_today: u32,
    daily_goal: u32,
    reps: u32,
    snooze_secs: u64,
}

#[derive(Clone, Serialize)]
pub struct SettingsView {
    settings: Settings,
    /// The language in effect, with "system" resolved: "en" or "tr".
    lang: &'static str,
    /// Dev build: minutes run as seconds and active hours are ignored.
    dev: bool,
}

#[derive(Clone, Copy)]
pub enum Pause {
    Hour,
    Today,
}

fn now() -> NaiveDateTime {
    Local::now().naive_local()
}

pub fn start<R: Runtime>(app: &AppHandle<R>) {
    let dir = app.path().app_data_dir().ok().filter(|dir| {
        std::fs::create_dir_all(dir)
            .inspect_err(|e| eprintln!("kalk: can't create {}: {e}", dir.display()))
            .is_ok()
    });
    let settings_path = dir.as_deref().map(settings::path);
    let settings = settings_path
        .as_deref()
        .map(settings::load)
        .unwrap_or_default();
    // Without a log the timer still works; only "done today" stays at 0.
    let db = dir.as_deref().and_then(|dir| {
        Db::open(&dir.join("kalk.db"))
            .inspect_err(|e| eprintln!("kalk: can't open log: {e}"))
            .ok()
    });

    let cfg = settings.timer_config(DEV);
    app.manage(Breaks {
        engine: Mutex::new(Engine::new(cfg)),
        settings: Mutex::new(settings.clone()),
        settings_path,
        db: db.map(Mutex::new),
        tooltip: Mutex::default(),
    });
    apply_side_effects(app, &settings);

    let app = app.clone();
    thread::spawn(move || {
        loop {
            on_tick(&app);
            // Re-read each time so a settings change takes effect.
            let tick = app.state::<Breaks>().engine.lock().unwrap().config().tick;
            thread::sleep(tick);
        }
    });
}

/// Applies the parts of the settings that live outside the timer.
fn apply_side_effects<R: Runtime>(app: &AppHandle<R>, settings: &Settings) {
    tray::set_language(app, settings.lang());
    // A dev build would register the debug binary to launch at login.
    #[cfg(windows)]
    if crate::store::packaged() {
        let enable = settings.launch_at_login;
        // Waits on WinRT calls, so keep it off the main thread.
        thread::spawn(move || {
            if let Err(e) = crate::store::set_launch_at_login(enable) {
                eprintln!("kalk: can't update launch at login: {e}");
            }
        });
        return;
    }
    if !DEV {
        use tauri_plugin_autostart::ManagerExt;
        let autolaunch = app.autolaunch();
        let result = if settings.launch_at_login {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        if let Err(e) = result {
            eprintln!("kalk: can't update launch at login: {e}");
        }
    }
}

fn on_tick<R: Runtime>(app: &AppHandle<R>) {
    let now = now();
    let breaks = app.state::<Breaks>();
    let event = breaks.engine.lock().unwrap().tick(now, idle::idle_time());
    match event {
        Some(Event::Remind) => {
            if breaks.settings.lock().unwrap().sound {
                sound::chime();
            }
            popup::show_reminder(app);
        }
        // Also re-shows the reminder in case it was dismissed somehow.
        Some(Event::Ask) => popup::show_reminder(app),
        None => {}
    }
    publish(app, now);
}

fn status<R: Runtime>(app: &AppHandle<R>, now: NaiveDateTime) -> Status {
    let breaks = app.state::<Breaks>();
    let engine = breaks.engine.lock().unwrap();
    let settings = breaks.settings.lock().unwrap();
    let done_today = breaks
        .db
        .as_ref()
        .and_then(|db| db.lock().unwrap().done_on(now.date()).ok())
        .unwrap_or(0);
    Status {
        phase: engine.phase(now),
        secs_left: engine.remaining().as_secs_f64().ceil() as u64,
        paused_until: engine
            .paused_until(now)
            .map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
        done_today,
        daily_goal: settings.daily_goal,
        reps: settings.reps,
        snooze_secs: engine.config().snooze.as_secs(),
    }
}

fn publish<R: Runtime>(app: &AppHandle<R>, now: NaiveDateTime) {
    let status = status(app, now);
    let _ = app.emit(STATUS_EVENT, &status);
    tray::sync_menu(app, status.phase == Phase::Paused);

    let breaks = app.state::<Breaks>();
    let tooltip = tooltip(breaks.settings.lock().unwrap().lang(), &status);
    let mut last = breaks.tooltip.lock().unwrap();
    if *last != tooltip {
        tray::set_tooltip(app, &tooltip);
        *last = tooltip;
    }
}

fn tooltip(lang: &str, s: &Status) -> String {
    match s.phase {
        Phase::Counting => i18n::next_break_in(lang, &i18n::duration(lang, s.secs_left)),
        Phase::Paused => {
            i18n::paused_until(lang, s.paused_until.as_deref().map_or("", |t| &t[11..16]))
        }
        Phase::OffHours => i18n::off_hours(lang),
        Phase::Reminding | Phase::Asking => i18n::time_for_squats(lang, s.reps),
    }
}

pub fn pause<R: Runtime>(app: &AppHandle<R>, pause: Pause) {
    let now = now();
    let until = match pause {
        Pause::Hour => now + Duration::from_secs(60 * 60),
        Pause::Today => now
            .date()
            .succ_opt()
            .unwrap_or(now.date())
            .and_time(NaiveTime::MIN),
    };
    app.state::<Breaks>()
        .engine
        .lock()
        .unwrap()
        .pause_until(until);
    publish(app, now);
}

pub fn resume<R: Runtime>(app: &AppHandle<R>) {
    app.state::<Breaks>().engine.lock().unwrap().resume();
    publish(app, now());
}

fn settings_view(settings: Settings) -> SettingsView {
    SettingsView {
        lang: settings.lang(),
        settings,
        dev: DEV,
    }
}

#[tauri::command]
pub fn get_status(app: AppHandle) -> Status {
    status(&app, now())
}

#[tauri::command]
pub fn answer(app: AppHandle, outcome: Outcome) {
    let now = now();
    let breaks = app.state::<Breaks>();
    if !breaks.engine.lock().unwrap().answer(outcome) {
        return;
    }
    let reps = breaks.settings.lock().unwrap().reps;
    if let Some(db) = &breaks.db
        && let Err(e) = db.lock().unwrap().log(now, outcome, reps)
    {
        eprintln!("kalk: can't log break: {e}");
    }
    popup::release(&app);
    publish(&app, now);
}

#[tauri::command]
pub fn resume_breaks(app: AppHandle) {
    resume(&app);
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> SettingsView {
    settings_view(app.state::<Breaks>().settings.lock().unwrap().clone())
}

#[tauri::command]
pub fn set_settings(app: AppHandle, settings: Settings) -> Result<SettingsView, String> {
    let settings = settings.sanitized();
    let breaks = app.state::<Breaks>();
    if let Some(path) = &breaks.settings_path {
        settings::save(path, &settings).map_err(|e| e.to_string())?;
    }
    breaks
        .engine
        .lock()
        .unwrap()
        .set_config(settings.timer_config(DEV));
    *breaks.settings.lock().unwrap() = settings.clone();
    apply_side_effects(&app, &settings);
    publish(&app, now());
    Ok(settings_view(settings))
}

#[tauri::command]
pub fn get_stats(app: AppHandle) -> Result<Stats, String> {
    let breaks = app.state::<Breaks>();
    let settings = breaks.settings.lock().unwrap().clone();
    let (done, total) = match &breaks.db {
        Some(db) => {
            let db = db.lock().unwrap();
            let done = db.done_per_day().map_err(|e| e.to_string())?;
            (done, db.total_squats().map_err(|e| e.to_string())?)
        }
        None => Default::default(),
    };
    let weekdays = settings.weekdays();
    let active_days = settings.active_hours.then_some(weekdays.as_slice());
    Ok(stats::build(
        &done,
        total,
        now().date(),
        settings.daily_goal,
        active_days,
    ))
}

#[tauri::command]
pub fn reset_stats(app: AppHandle) -> Result<(), String> {
    if let Some(db) = &app.state::<Breaks>().db {
        db.lock().unwrap().clear().map_err(|e| e.to_string())?;
    }
    publish(&app, now());
    Ok(())
}
