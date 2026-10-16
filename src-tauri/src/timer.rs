//! The break timer: pure logic, driven by `tick` with the wall-clock time and
//! the system idle time, so it can be tested with a fake clock.
//!
//! - Active time accumulates only while the user is at the computer.
//! - Idle ≥ threshold, or a wall-clock jump that long (sleep / lock), resets it:
//!   the user already got up.
//! - Outside active hours, or while paused, the timer doesn't run.
//! - Active time ≥ interval opens a reminder, which stays open until answered.
//!   If the user walks away from an open reminder, they're asked on return
//!   whether they did the squats.

use std::time::Duration;

use chrono::{Datelike, NaiveDateTime, NaiveTime, Weekday};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct ActiveHours {
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub days: Vec<Weekday>,
}

impl ActiveHours {
    fn contains(&self, t: NaiveDateTime) -> bool {
        self.days.contains(&t.weekday()) && t.time() >= self.start && t.time() < self.end
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub interval: Duration,
    pub idle_threshold: Duration,
    pub snooze: Duration,
    /// How often `tick` is called. A gap between ticks counts as active time
    /// only up to a few ticks, so a stalled loop can't fire a reminder early.
    pub tick: Duration,
    /// `None` runs the timer around the clock.
    pub active_hours: Option<ActiveHours>,
}

impl Config {
    pub fn release() -> Self {
        let h = |h| NaiveTime::from_hms_opt(h, 0, 0).unwrap();
        Config {
            interval: Duration::from_secs(45 * 60),
            idle_threshold: Duration::from_secs(10 * 60),
            snooze: Duration::from_secs(5 * 60),
            tick: Duration::from_secs(10),
            active_hours: Some(ActiveHours {
                start: h(9),
                end: h(18),
                days: vec![
                    Weekday::Mon,
                    Weekday::Tue,
                    Weekday::Wed,
                    Weekday::Thu,
                    Weekday::Fri,
                ],
            }),
        }
    }

    /// Seconds instead of minutes, and no active hours, so a full cycle can be
    /// watched locally at any time of day.
    pub fn dev() -> Self {
        Config {
            interval: Duration::from_secs(45),
            idle_threshold: Duration::from_secs(30),
            snooze: Duration::from_secs(15),
            tick: Duration::from_secs(1),
            active_hours: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Counting,
    Paused,
    OffHours,
    /// Reminder is open, waiting for Done / Snooze / Skip.
    Reminding,
    /// The user left an open reminder and came back: "Did you do your squats?"
    Asking,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Remind,
    Ask,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Done,
    Skipped,
    Snoozed,
    Missed,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Done => "done",
            Outcome::Skipped => "skipped",
            Outcome::Snoozed => "snoozed",
            Outcome::Missed => "missed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reminder {
    None,
    Open,
    /// Open, and the user has been idle past the threshold since.
    Away,
    Asking,
}

pub struct Engine {
    pub cfg: Config,
    active: Duration,
    last_tick: Option<NaiveDateTime>,
    reminder: Reminder,
    paused_until: Option<NaiveDateTime>,
}

impl Engine {
    pub fn new(cfg: Config) -> Self {
        Engine {
            cfg,
            active: Duration::ZERO,
            last_tick: None,
            reminder: Reminder::None,
            paused_until: None,
        }
    }

    pub fn tick(&mut self, now: NaiveDateTime, idle: Duration) -> Option<Event> {
        let gap = self
            .last_tick
            .and_then(|last| (now - last).to_std().ok())
            .unwrap_or(Duration::ZERO);
        self.last_tick = Some(now);
        let away = idle >= self.cfg.idle_threshold || gap >= self.cfg.idle_threshold;

        match self.reminder {
            Reminder::Open if away => {
                self.reminder = Reminder::Away;
                return None;
            }
            Reminder::Away if !away => {
                self.reminder = Reminder::Asking;
                return Some(Event::Ask);
            }
            Reminder::Open | Reminder::Away | Reminder::Asking => return None,
            Reminder::None => {}
        }

        if self.paused_until.is_some_and(|until| now < until) {
            return None;
        }
        self.paused_until = None;

        if away || !self.in_active_hours(now) {
            self.active = Duration::ZERO;
            return None;
        }

        self.active += gap.min(self.cfg.tick * 3);
        if self.active >= self.cfg.interval {
            self.reminder = Reminder::Open;
            return Some(Event::Remind);
        }
        None
    }

    /// Closes the open reminder. Returns false (and changes nothing) if there
    /// is no reminder to answer.
    pub fn answer(&mut self, outcome: Outcome) -> bool {
        if self.reminder == Reminder::None {
            return false;
        }
        self.reminder = Reminder::None;
        self.active = match outcome {
            Outcome::Snoozed => self.cfg.interval.saturating_sub(self.cfg.snooze),
            _ => Duration::ZERO,
        };
        true
    }

    pub fn pause_until(&mut self, until: NaiveDateTime) {
        self.paused_until = Some(until);
        self.active = Duration::ZERO;
    }

    pub fn resume(&mut self) {
        self.paused_until = None;
    }

    pub fn paused_until(&self, now: NaiveDateTime) -> Option<NaiveDateTime> {
        self.paused_until.filter(|&until| now < until)
    }

    pub fn phase(&self, now: NaiveDateTime) -> Phase {
        match self.reminder {
            Reminder::Open | Reminder::Away => Phase::Reminding,
            Reminder::Asking => Phase::Asking,
            Reminder::None if self.paused_until(now).is_some() => Phase::Paused,
            Reminder::None if !self.in_active_hours(now) => Phase::OffHours,
            Reminder::None => Phase::Counting,
        }
    }

    pub fn remaining(&self) -> Duration {
        self.cfg.interval.saturating_sub(self.active)
    }

    fn in_active_hours(&self, now: NaiveDateTime) -> bool {
        self.cfg.active_hours.as_ref().is_none_or(|h| h.contains(now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    const MIN: Duration = Duration::from_secs(60);
    const BUSY: Duration = Duration::from_secs(1);

    fn at(day: u32, h: u32, m: u32) -> NaiveDateTime {
        // October 2026: the 12th is a Monday, the 17th a Saturday.
        NaiveDate::from_ymd_opt(2026, 10, day)
            .unwrap()
            .and_hms_opt(h, m, 0)
            .unwrap()
    }

    /// Release config ticking once a minute, which keeps the tests short.
    fn engine() -> Engine {
        Engine::new(Config {
            tick: MIN,
            ..Config::release()
        })
    }

    /// Ticks once a minute from `start` for `mins` minutes with constant idle
    /// time; returns the events and the time of the last tick.
    fn run(
        e: &mut Engine,
        start: NaiveDateTime,
        mins: i64,
        idle: Duration,
    ) -> (Vec<Event>, NaiveDateTime) {
        let mut events = vec![];
        let mut now = start;
        for i in 0..=mins {
            now = start + chrono::Duration::minutes(i);
            events.extend(e.tick(now, idle));
        }
        (events, now)
    }

    #[test]
    fn reminds_after_interval_of_activity() {
        let mut e = engine();
        let (events, _) = run(&mut e, at(12, 10, 0), 44, BUSY);
        assert!(events.is_empty());
        assert_eq!(e.remaining(), MIN);
        let (events, now) = run(&mut e, at(12, 10, 45), 0, BUSY);
        assert_eq!(events, vec![Event::Remind]);
        assert_eq!(e.phase(now), Phase::Reminding);
    }

    #[test]
    fn idle_past_threshold_resets() {
        let mut e = engine();
        run(&mut e, at(12, 10, 0), 30, BUSY);
        e.tick(at(12, 10, 31), 10 * MIN);
        assert_eq!(e.remaining(), 45 * MIN);
        let (events, _) = run(&mut e, at(12, 10, 32), 43, BUSY);
        assert!(events.is_empty());
    }

    #[test]
    fn short_idle_keeps_counting() {
        let mut e = engine();
        run(&mut e, at(12, 10, 0), 20, BUSY);
        e.tick(at(12, 10, 21), 9 * MIN);
        assert_eq!(e.remaining(), 24 * MIN);
    }

    #[test]
    fn wall_clock_jump_resets() {
        let mut e = engine();
        run(&mut e, at(12, 10, 0), 30, BUSY);
        // Laptop slept for an hour; on wake the user just touched the keyboard.
        e.tick(at(12, 11, 31), BUSY);
        assert_eq!(e.remaining(), 45 * MIN);
    }

    #[test]
    fn short_gap_counts_only_a_few_ticks() {
        let mut e = engine();
        e.tick(at(12, 10, 0), BUSY);
        e.tick(at(12, 10, 8), BUSY);
        assert_eq!(e.remaining(), 42 * MIN);
    }

    #[test]
    fn does_not_run_outside_active_hours() {
        let mut e = engine();
        let (events, now) = run(&mut e, at(12, 17, 30), 60, BUSY);
        assert!(events.is_empty());
        assert_eq!(e.phase(now), Phase::OffHours);
        let (events, _) = run(&mut e, at(17, 10, 0), 60, BUSY);
        assert!(events.is_empty(), "Saturday is off");
    }

    #[test]
    fn done_and_skip_restart_the_interval() {
        for outcome in [Outcome::Done, Outcome::Skipped] {
            let mut e = engine();
            run(&mut e, at(12, 10, 0), 45, BUSY);
            assert!(e.answer(outcome));
            assert_eq!(e.phase(at(12, 10, 45)), Phase::Counting);
            assert_eq!(e.remaining(), 45 * MIN);
        }
    }

    #[test]
    fn snooze_reminds_again_after_snooze_time() {
        let mut e = engine();
        run(&mut e, at(12, 10, 0), 45, BUSY);
        assert!(e.answer(Outcome::Snoozed));
        let (events, _) = run(&mut e, at(12, 10, 46), 3, BUSY);
        assert!(events.is_empty());
        let (events, _) = run(&mut e, at(12, 10, 50), 0, BUSY);
        assert_eq!(events, vec![Event::Remind]);
    }

    #[test]
    fn answer_without_reminder_is_ignored() {
        let mut e = engine();
        run(&mut e, at(12, 10, 0), 10, BUSY);
        assert!(!e.answer(Outcome::Done));
        assert_eq!(e.remaining(), 35 * MIN);
    }

    #[test]
    fn reminder_stays_open_while_working() {
        let mut e = engine();
        run(&mut e, at(12, 10, 0), 45, BUSY);
        let (events, now) = run(&mut e, at(12, 10, 46), 30, BUSY);
        assert!(events.is_empty());
        assert_eq!(e.phase(now), Phase::Reminding);
    }

    #[test]
    fn walking_away_from_reminder_asks_on_return() {
        let mut e = engine();
        run(&mut e, at(12, 10, 0), 45, BUSY);
        let (events, now) = run(&mut e, at(12, 10, 46), 15, 12 * MIN);
        assert!(events.is_empty());
        assert_eq!(e.phase(now), Phase::Reminding);
        assert_eq!(e.tick(at(12, 11, 2), BUSY), Some(Event::Ask));
        assert_eq!(e.phase(at(12, 11, 2)), Phase::Asking);
        assert!(e.answer(Outcome::Missed));
        assert_eq!(e.phase(at(12, 11, 2)), Phase::Counting);
    }

    #[test]
    fn pause_stops_and_restarts_fresh() {
        let mut e = engine();
        run(&mut e, at(12, 10, 0), 30, BUSY);
        e.pause_until(at(12, 11, 30));
        let (events, now) = run(&mut e, at(12, 10, 31), 58, BUSY);
        assert!(events.is_empty());
        assert_eq!(e.phase(now), Phase::Paused);
        e.tick(at(12, 11, 30), BUSY);
        assert_eq!(e.phase(at(12, 11, 30)), Phase::Counting);
        assert_eq!(e.remaining(), 44 * MIN);
    }

    #[test]
    fn resume_ends_pause() {
        let mut e = engine();
        e.pause_until(at(13, 0, 0));
        assert_eq!(e.phase(at(12, 10, 0)), Phase::Paused);
        e.resume();
        assert_eq!(e.phase(at(12, 10, 0)), Phase::Counting);
    }
}
