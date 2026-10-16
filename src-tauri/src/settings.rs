//! User settings, stored as JSON in the app data dir.

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{NaiveTime, Weekday};
use serde::{Deserialize, Serialize};

use crate::timer::{ActiveHours, Config};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    System,
    En,
    Tr,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub interval_mins: u32,
    pub reps: u32,
    /// Done breaks per day that count towards the streak.
    pub daily_goal: u32,
    pub idle_mins: u32,
    pub snooze_mins: u32,
    pub active_hours: bool,
    /// "HH:MM", local time.
    pub start: String,
    pub end: String,
    /// 0 = Monday … 6 = Sunday.
    pub days: Vec<u8>,
    pub launch_at_login: bool,
    pub sound: bool,
    pub language: Language,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            interval_mins: 45,
            reps: 10,
            daily_goal: 8,
            idle_mins: 10,
            snooze_mins: 5,
            active_hours: true,
            start: "09:00".into(),
            end: "18:00".into(),
            days: vec![0, 1, 2, 3, 4],
            launch_at_login: true,
            sound: true,
            language: Language::System,
        }
    }
}

fn parse_time(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M").ok()
}

impl Settings {
    /// Clamps values into sane ranges so a hand-edited file can't break the
    /// timer.
    pub fn sanitized(mut self) -> Self {
        let d = Settings::default();
        self.interval_mins = self.interval_mins.clamp(1, 240);
        self.reps = self.reps.clamp(1, 100);
        self.daily_goal = self.daily_goal.clamp(1, 50);
        self.idle_mins = self.idle_mins.clamp(1, 120);
        self.snooze_mins = self.snooze_mins.clamp(1, 60);
        if parse_time(&self.start).is_none() {
            self.start = d.start;
        }
        if parse_time(&self.end).is_none() {
            self.end = d.end;
        }
        self.days.retain(|&day| day < 7);
        self.days.sort_unstable();
        self.days.dedup();
        self
    }

    /// In dev builds every minute runs as a second, so a full cycle can be
    /// watched locally, and active hours are ignored.
    pub fn timer_config(&self, dev: bool) -> Config {
        let unit = if dev { 1 } else { 60 };
        let mins = |m: u32| Duration::from_secs(u64::from(m) * unit);
        // Under ~30 s, reading the reminder without moving the mouse would
        // already count as walking away.
        let idle = if dev {
            mins(self.idle_mins).max(Duration::from_secs(30))
        } else {
            mins(self.idle_mins)
        };
        let active_hours = (self.active_hours && !dev).then(|| ActiveHours {
            start: parse_time(&self.start).unwrap_or(NaiveTime::MIN),
            end: parse_time(&self.end).unwrap_or(NaiveTime::MIN),
            days: self.weekdays(),
        });
        Config {
            interval: mins(self.interval_mins),
            idle_threshold: idle,
            snooze: mins(self.snooze_mins),
            tick: Duration::from_secs(if dev { 1 } else { 10 }),
            active_hours,
        }
    }

    pub fn weekdays(&self) -> Vec<Weekday> {
        self.days
            .iter()
            .filter_map(|&d| Weekday::try_from(d).ok())
            .collect()
    }

    /// "tr" or "en".
    pub fn lang(&self) -> &'static str {
        match self.language {
            Language::En => "en",
            Language::Tr => "tr",
            Language::System => {
                let tr = sys_locale::get_locale()
                    .is_some_and(|l| l.to_ascii_lowercase().starts_with("tr"));
                if tr { "tr" } else { "en" }
            }
        }
    }
}

pub fn path(dir: &Path) -> PathBuf {
    dir.join("settings.json")
}

pub fn load(path: &Path) -> Settings {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
        .unwrap_or_default()
        .sanitized()
}

pub fn save(path: &Path, settings: &Settings) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(settings).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let s: Settings = serde_json::from_str(r#"{"reps": 15}"#).unwrap();
        assert_eq!(s.reps, 15);
        assert_eq!(s.interval_mins, 45);
    }

    #[test]
    fn sanitizes_bad_values() {
        let s = Settings {
            interval_mins: 0,
            start: "nonsense".into(),
            days: vec![4, 9, 1, 4],
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(s.interval_mins, 1);
        assert_eq!(s.start, "09:00");
        assert_eq!(s.days, vec![1, 4]);
    }

    #[test]
    fn release_config_uses_minutes_and_active_hours() {
        let c = Settings::default().timer_config(false);
        assert_eq!(c.interval, Duration::from_secs(45 * 60));
        assert_eq!(c.active_hours.unwrap().days.len(), 5);
    }

    #[test]
    fn dev_config_uses_seconds() {
        let c = Settings::default().timer_config(true);
        assert_eq!(c.interval, Duration::from_secs(45));
        assert_eq!(c.idle_threshold, Duration::from_secs(30));
        assert!(c.active_hours.is_none());
    }
}
