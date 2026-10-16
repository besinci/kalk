//! Local SQLite log of every reminder and how it was answered.

use std::collections::HashMap;
use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use rusqlite::{Connection, params};

use crate::timer::Outcome;

pub struct Db(Connection);

/// Schema changes, applied in order; `PRAGMA user_version` counts how many
/// have run.
const MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS breaks (
        id      INTEGER PRIMARY KEY,
        at      TEXT NOT NULL,  -- local time, ISO 8601
        outcome TEXT NOT NULL   -- done | skipped | snoozed | missed
    );
    CREATE INDEX IF NOT EXISTS breaks_at ON breaks (at);",
    // Reps per break, so total squats stay right when the setting changes.
    "ALTER TABLE breaks ADD COLUMN reps INTEGER NOT NULL DEFAULT 10;",
];

fn day_key(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

impl Db {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        Self::init(Connection::open(path)?)
    }

    #[cfg(test)]
    fn in_memory() -> rusqlite::Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Self> {
        let version: u32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(version as usize) {
            conn.execute_batch(&format!(
                "BEGIN; {sql} PRAGMA user_version = {}; COMMIT;",
                i + 1
            ))?;
        }
        Ok(Db(conn))
    }

    pub fn log(&self, at: NaiveDateTime, outcome: Outcome, reps: u32) -> rusqlite::Result<()> {
        self.0.execute(
            "INSERT INTO breaks (at, outcome, reps) VALUES (?1, ?2, ?3)",
            params![
                at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                outcome.as_str(),
                reps
            ],
        )?;
        Ok(())
    }

    pub fn clear(&self) -> rusqlite::Result<()> {
        self.0.execute("DELETE FROM breaks", [])?;
        Ok(())
    }

    pub fn done_on(&self, day: NaiveDate) -> rusqlite::Result<u32> {
        self.0.query_row(
            "SELECT count(*) FROM breaks WHERE outcome = 'done' AND substr(at, 1, 10) = ?1",
            params![day_key(day)],
            |row| row.get(0),
        )
    }

    /// Done breaks per day, for every day that has any.
    pub fn done_per_day(&self) -> rusqlite::Result<HashMap<NaiveDate, u32>> {
        let mut stmt = self.0.prepare(
            "SELECT substr(at, 1, 10), count(*) FROM breaks
             WHERE outcome = 'done' GROUP BY 1",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })?;
        let mut days = HashMap::new();
        for row in rows {
            let (day, n) = row?;
            if let Ok(day) = NaiveDate::parse_from_str(&day, "%Y-%m-%d") {
                days.insert(day, n);
            }
        }
        Ok(days)
    }

    pub fn total_squats(&self) -> rusqlite::Result<u64> {
        let total: i64 = self.0.query_row(
            "SELECT coalesce(sum(reps), 0) FROM breaks WHERE outcome = 'done'",
            [],
            |row| row.get(0),
        )?;
        Ok(total.max(0) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(d: NaiveDate, h: u32) -> NaiveDateTime {
        d.and_hms_opt(h, 0, 0).unwrap()
    }

    #[test]
    fn counts_done_breaks_per_day() {
        let db = Db::in_memory().unwrap();
        let day = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
        let next = day.succ_opt().unwrap();
        db.log(at(day, 10), Outcome::Done, 10).unwrap();
        db.log(at(day, 11), Outcome::Skipped, 10).unwrap();
        db.log(at(day, 12), Outcome::Done, 15).unwrap();
        db.log(at(next, 10), Outcome::Done, 10).unwrap();
        assert_eq!(db.done_on(day).unwrap(), 2);
        assert_eq!(
            db.done_per_day().unwrap(),
            HashMap::from([(day, 2), (next, 1)])
        );
        assert_eq!(db.total_squats().unwrap(), 35);
    }

    #[test]
    fn migrates_a_version_1_log() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(MIGRATIONS[0]).unwrap();
        conn.execute_batch(
            "PRAGMA user_version = 1;
             INSERT INTO breaks (at, outcome) VALUES ('2026-10-16T10:00:00', 'done');",
        )
        .unwrap();
        let db = Db::init(conn).unwrap();
        assert_eq!(db.total_squats().unwrap(), 10);
    }
}
