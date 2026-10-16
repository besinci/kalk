//! The stats view: today, streak, the last 7 days and total squats.

use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Weekday};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Day {
    /// "YYYY-MM-DD"
    date: String,
    done: u32,
}

#[derive(Clone, Serialize)]
pub struct Stats {
    today: u32,
    goal: u32,
    streak: u32,
    /// Oldest first, ending today.
    week: Vec<Day>,
    total_squats: u64,
}

pub fn build(
    done: &HashMap<NaiveDate, u32>,
    total_squats: u64,
    today: NaiveDate,
    goal: u32,
    active_days: Option<&[Weekday]>,
) -> Stats {
    let week = (0..7)
        .rev()
        .filter_map(|back| today.checked_sub_days(chrono::Days::new(back)))
        .map(|d| Day {
            date: d.format("%Y-%m-%d").to_string(),
            done: done.get(&d).copied().unwrap_or(0),
        })
        .collect();
    Stats {
        today: done.get(&today).copied().unwrap_or(0),
        goal,
        streak: streak(done, today, goal, active_days),
        week,
        total_squats,
    }
}

/// Consecutive active days, back from today, that reached the goal. Days off
/// (`active_days`, if set) neither count nor break the streak, and today only
/// breaks it once it's over.
pub fn streak(
    done: &HashMap<NaiveDate, u32>,
    today: NaiveDate,
    goal: u32,
    active_days: Option<&[Weekday]>,
) -> u32 {
    let reached = |d: &NaiveDate| done.get(d).is_some_and(|&n| n >= goal);
    let is_active = |d: &NaiveDate| active_days.is_none_or(|days| days.contains(&d.weekday()));
    // Nothing to find beyond the oldest logged day.
    let Some(&oldest) = done.keys().min() else {
        return 0;
    };

    let mut streak = 0;
    let mut day = today;
    if reached(&day) {
        streak += 1;
    }
    while let Some(prev) = day.pred_opt() {
        day = prev;
        if day < oldest {
            break;
        }
        if !is_active(&day) {
            continue;
        }
        if !reached(&day) {
            break;
        }
        streak += 1;
    }
    streak
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(day: u32) -> NaiveDate {
        // October 2026: the 12th is a Monday.
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    const WEEKDAYS: &[Weekday] = &[
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ];

    #[test]
    fn counts_back_to_first_missed_day() {
        let done = HashMap::from([(d(13), 8), (d(14), 3), (d(15), 8), (d(16), 9)]);
        assert_eq!(streak(&done, d(16), 8, None), 2);
    }

    #[test]
    fn today_in_progress_does_not_break_it() {
        let done = HashMap::from([(d(14), 8), (d(15), 8), (d(16), 2)]);
        assert_eq!(streak(&done, d(16), 8, None), 2);
    }

    #[test]
    fn weekends_off_are_skipped() {
        // Fri 9th, Mon 12th, Tue 13th reached; the weekend in between is off.
        let done = HashMap::from([(d(9), 8), (d(12), 8), (d(13), 8)]);
        assert_eq!(streak(&done, d(13), 8, Some(WEEKDAYS)), 3);
        assert_eq!(streak(&done, d(13), 8, None), 2);
    }

    #[test]
    fn empty_log_has_no_streak() {
        assert_eq!(streak(&HashMap::new(), d(16), 8, None), 0);
    }

    #[test]
    fn week_ends_today_oldest_first() {
        let done = HashMap::from([(d(10), 1), (d(16), 4)]);
        let s = build(&done, 50, d(16), 8, None);
        let dates: Vec<_> = s.week.iter().map(|day| day.date.as_str()).collect();
        assert_eq!(dates.first(), Some(&"2026-10-10"));
        assert_eq!(dates.last(), Some(&"2026-10-16"));
        assert_eq!(s.week[0].done, 1);
        assert_eq!(s.today, 4);
    }
}
