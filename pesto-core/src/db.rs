use std::{
    fs, iter,
    path::{Path, PathBuf},
    time::Duration,
};

use color_eyre::eyre::{self, Context};
use norcina::Event;
use rusqlite::{Connection, types::ValueRef};

use crate::{
    event::{CustomEvent, MaybeCustomEvent, Session},
    solve::{Penalty, Solve},
};

#[derive(Debug)]
pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new() -> eyre::Result<Self> {
        Self::new_at_path(None::<PathBuf>)
    }

    pub fn new_at_path(path: Option<impl AsRef<Path>>) -> eyre::Result<Self> {
        let conn = if let Some(path) = path {
            fs::create_dir_all(path.as_ref().parent().unwrap())?;
            Connection::open(path)
        } else if cfg!(debug_assertions) {
            Connection::open("./main.db")
        } else {
            let proj_dirs = proj_dirs();
            let dir = proj_dirs.data_dir();
            fs::create_dir_all(&dir)?;
            Connection::open(dir.join("main.db"))
        }?;

        conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS solve (
            id INTEGER PRIMARY KEY,
            --- Amount of time it took to solve
            time_ms INTEGER NOT NULL,
            --- Datetime when the solve ended, according to [RFC 8536](https://datatracker.ietf.org/doc/html/rfc8536)
            end_date INTEGER NOT NULL,
            scramble TEXT NOT NULL,
            penalty INTEGER NOT NULL,
            event_id INTEGER NOT NULL,
            session_id INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS custom_event (
            --- Cannot be 0-16 (inclusive), minimum allowed value is 17. 0-16 indicate official WCA events.
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            --- Between 0-16, index of an official WCA event. Can be null.
            scramble_type INTEGER
        );
        CREATE TABLE IF NOT EXISTS custom_session (
            --- Cannot be 0 in the database. An index of 0 refers to the 'main' session.
            id INTEGER NOT NULL,
            name TEXT NOT NULL,
            event_id INTEGER NOT NULL,
            PRIMARY KEY (id, event_id)
        );
        COMMIT;",
    )
    .wrap_err("Couldn't initialize database")?;

        Ok(Self { conn })
    }

    pub fn get_events(&mut self) -> eyre::Result<Vec<MaybeCustomEvent>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, scramble_type FROM custom_event")?;

        let custom_events = stmt
            .query_map([], |row| {
                Ok(CustomEvent {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    scramble_type: official_event_from_sql(row.get_ref(2)?)?,
                })
            })?
            .map(|event| event.map(MaybeCustomEvent::Unofficial).map_err(Into::into));

        let official_events = Event::ALL
            .into_iter()
            .map(|event| Ok(MaybeCustomEvent::Official(event)));

        official_events.chain(custom_events).collect()
    }

    pub fn get_sessions_of_event(
        &mut self,
        event: &MaybeCustomEvent,
    ) -> eyre::Result<Vec<Session>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM custom_session WHERE event_id = $1")?;

        let iter = stmt.query_map([event.id()], |row| {
            Ok(Session::Custom {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        iter::once(Ok(Session::Main))
            .chain(iter.map(|v| v.map_err(Into::into)))
            .collect()
    }

    pub fn get_events_and_sessions(
        &mut self,
    ) -> eyre::Result<Vec<(MaybeCustomEvent, Vec<Session>)>> {
        let events = self.get_events()?;
        events
            .into_iter()
            .map(|event| {
                let sessions = self.get_sessions_of_event(&event)?;
                Ok((event, sessions))
            })
            .collect()
    }

    pub fn insert_solve(
        &self,
        solve: Solve,
        event: &MaybeCustomEvent,
        session: &Session,
    ) -> eyre::Result<()> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO solve
                (time_ms, end_date, scramble, penalty, event_id, session_id)
            VALUES ($1,   $2,       $3,       $4,      $5,       $6)",
        )?;

        stmt.execute((
            solve.time.as_millis() as u64,
            solve.end_date.to_string(),
            solve.scramble,
            solve.penalty.index(),
            event.id(),
            session.id(),
        ))?;

        Ok(())
    }

    pub fn get_solves(
        &mut self,
        event: &MaybeCustomEvent,
        session: &Session,
    ) -> eyre::Result<Vec<Solve>> {
        let mut stmt = self.conn.prepare("SELECT time_ms, end_date, scramble, penalty FROM solve WHERE event_id = $1 AND session_id = $2")?;
        let iter = stmt.query_map([event.id(), session.id()], |row| {
            Ok(Solve {
                time: Duration::from_millis(row.get(0)?),
                end_date: row
                    .get::<_, String>(1)?
                    .parse()
                    .expect("valid RFC 8536 format in db"),
                scramble: row.get(2)?,
                penalty: Penalty::from_index(row.get(3)?).expect("Valid penalty index in db"),
            })
        })?;

        iter.map(|v| v.map_err(Into::into)).collect()
    }
}

fn proj_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("com", "odilf", "pesto")
        .expect("Have a project directory available, in either Windows, MacOS or Linux.")
}

fn official_event_from_sql(value: ValueRef<'_>) -> rusqlite::Result<Option<Event>> {
    Ok(match value {
        ValueRef::Null => None,
        ValueRef::Integer(id) => Some(Event::ALL[id as usize]),
        value => {
            return Err(rusqlite::Error::InvalidColumnType(
                2,
                "event_id".to_string(),
                value.data_type(),
            ));
        }
    })
}
