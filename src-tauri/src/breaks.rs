//! Runs the timer in the background and wires it to the popup, the tray and
//! the log. The popup gets a `status` event on every tick.

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use chrono::{Local, NaiveDateTime, NaiveTime};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::db::Db;
use crate::timer::{Config, Engine, Event, Outcome, Phase};
use crate::{idle, popup, tray};

pub const STATUS_EVENT: &str = "status";
/// TODO(milestone 5): make reps a setting.
const REPS: u32 = 10;

pub struct Breaks {
    engine: Mutex<Engine>,
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
    reps: u32,
    snooze_secs: u64,
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
    let cfg = if cfg!(debug_assertions) {
        Config::dev()
    } else {
        Config::release()
    };
    let tick = cfg.tick;
    app.manage(Breaks {
        engine: Mutex::new(Engine::new(cfg)),
        db: open_db(app).map(Mutex::new),
        tooltip: Mutex::default(),
    });

    let app = app.clone();
    thread::spawn(move || {
        loop {
            on_tick(&app);
            thread::sleep(tick);
        }
    });
}

fn open_db<R: Runtime>(app: &AppHandle<R>) -> Option<Db> {
    let dir = app.path().app_data_dir().ok()?;
    let opened = std::fs::create_dir_all(&dir)
        .map_err(|e| e.to_string())
        .and_then(|()| Db::open(&dir.join("kalk.db")).map_err(|e| e.to_string()));
    // Without a log the timer still works; only "done today" stays at 0.
    opened.inspect_err(|e| eprintln!("kalk: can't open log: {e}")).ok()
}

fn on_tick<R: Runtime>(app: &AppHandle<R>) {
    let now = now();
    let event = app
        .state::<Breaks>()
        .engine
        .lock()
        .unwrap()
        .tick(now, idle::idle_time());
    match event {
        // Ask also re-shows the reminder in case it was dismissed somehow.
        Some(Event::Remind | Event::Ask) => popup::show_reminder(app),
        None => {}
    }
    publish(app, now);
}

fn status<R: Runtime>(app: &AppHandle<R>, now: NaiveDateTime) -> Status {
    let breaks = app.state::<Breaks>();
    let engine = breaks.engine.lock().unwrap();
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
        reps: REPS,
        snooze_secs: engine.cfg.snooze.as_secs(),
    }
}

fn publish<R: Runtime>(app: &AppHandle<R>, now: NaiveDateTime) {
    let status = status(app, now);
    let _ = app.emit(STATUS_EVENT, &status);
    tray::sync_menu(app, status.phase == Phase::Paused);

    let tooltip = tooltip(&status);
    let breaks = app.state::<Breaks>();
    let mut last = breaks.tooltip.lock().unwrap();
    if *last != tooltip {
        tray::set_tooltip(app, &tooltip);
        *last = tooltip;
    }
}

fn tooltip(s: &Status) -> String {
    // TODO(milestone 5): localise.
    match s.phase {
        Phase::Counting => format!("Kalk — next break in {}", fmt_duration(s.secs_left)),
        Phase::Paused => match &s.paused_until {
            Some(until) => format!("Kalk — paused until {}", &until[11..16]),
            None => "Kalk — paused".into(),
        },
        Phase::OffHours => "Kalk — outside active hours".into(),
        Phase::Reminding | Phase::Asking => format!("Kalk — time for {} squats!", s.reps),
    }
}

/// "23 min", or "0:42" under a minute.
fn fmt_duration(secs: u64) -> String {
    if secs >= 60 {
        format!("{} min", secs.div_ceil(60))
    } else {
        format!("0:{secs:02}")
    }
}

pub fn pause<R: Runtime>(app: &AppHandle<R>, pause: Pause) {
    let now = now();
    let until = match pause {
        Pause::Hour => now + Duration::from_secs(60 * 60),
        Pause::Today => now.date().succ_opt().unwrap_or(now.date()).and_time(NaiveTime::MIN),
    };
    app.state::<Breaks>().engine.lock().unwrap().pause_until(until);
    publish(app, now);
}

pub fn resume<R: Runtime>(app: &AppHandle<R>) {
    app.state::<Breaks>().engine.lock().unwrap().resume();
    publish(app, now());
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
    if let Some(db) = &breaks.db
        && let Err(e) = db.lock().unwrap().log(now, outcome)
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
