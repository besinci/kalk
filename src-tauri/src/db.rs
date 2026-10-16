//! Local SQLite log of every reminder and how it was answered.

use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use rusqlite::{Connection, params};

use crate::timer::Outcome;

pub struct Db(Connection);

impl Db {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        Self::init(Connection::open(path)?)
    }

    #[cfg(test)]
    fn in_memory() -> rusqlite::Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS breaks (
                id      INTEGER PRIMARY KEY,
                at      TEXT NOT NULL,  -- local time, ISO 8601
                outcome TEXT NOT NULL   -- done | skipped | snoozed | missed
            );
            CREATE INDEX IF NOT EXISTS breaks_at ON breaks (at);",
        )?;
        Ok(Db(conn))
    }

    pub fn log(&self, at: NaiveDateTime, outcome: Outcome) -> rusqlite::Result<()> {
        self.0.execute(
            "INSERT INTO breaks (at, outcome) VALUES (?1, ?2)",
            params![at.format("%Y-%m-%dT%H:%M:%S").to_string(), outcome.as_str()],
        )?;
        Ok(())
    }

    pub fn done_on(&self, day: NaiveDate) -> rusqlite::Result<u32> {
        self.0.query_row(
            "SELECT count(*) FROM breaks WHERE outcome = 'done' AND substr(at, 1, 10) = ?1",
            params![day.format("%Y-%m-%d").to_string()],
            |row| row.get(0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_done_breaks_per_day() {
        let db = Db::in_memory().unwrap();
        let day = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
        let at = |d: NaiveDate, h| d.and_hms_opt(h, 0, 0).unwrap();
        db.log(at(day, 10), Outcome::Done).unwrap();
        db.log(at(day, 11), Outcome::Skipped).unwrap();
        db.log(at(day, 12), Outcome::Done).unwrap();
        db.log(at(day.succ_opt().unwrap(), 10), Outcome::Done).unwrap();
        assert_eq!(db.done_on(day).unwrap(), 2);
    }
}
