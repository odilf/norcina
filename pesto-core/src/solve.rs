use color_eyre::eyre;
use jiff::{Zoned, civil::DateTime};
use std::{
    fmt,
    time::{Duration, SystemTime},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Penalty {
    /// No penalty.
    None,
    /// +2, generally because the timer stopped when the cube was one move from being solved.
    Plus2,
    /// Timer stopped and the cube was not finished.
    DNF,
}

impl Penalty {
    pub const fn index(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Plus2 => 1,
            Self::DNF => 2,
        }
    }

    pub fn from_index(index: u8) -> eyre::Result<Penalty> {
        Ok(match index {
            0 => Penalty::None,
            1 => Penalty::Plus2,
            2 => Penalty::DNF,
            _ => eyre::bail!("Invalid penalty index"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Solve {
    /// The time that the solve took.
    pub time: Duration,
    /// The moment the solve was finished.
    pub end_date: Zoned,
    pub scramble: String,
    pub penalty: Penalty,
}

impl Solve {
    pub fn new(time: Duration, scramble: String) -> Self {
        Self {
            time,
            end_date: Zoned::now(),
            scramble,
            penalty: Penalty::None,
        }
    }
    /// The moment the solve was started.
    pub fn start_date(&self) -> Zoned {
        self.end_date.checked_sub(self.time).unwrap()
    }
}

impl fmt::Display for Solve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_duration_text(f, self.time)?;

        match self.penalty {
            Penalty::None => (),
            Penalty::Plus2 => write!(f, " (+2)")?,
            Penalty::DNF => write!(f, " (DNF)")?,
        }

        Ok(())
    }
}

fn format_duration_text(f: &mut fmt::Formatter<'_>, duration: Duration) -> fmt::Result {
    if duration < Duration::from_secs(60) {
        write!(
            f,
            "{:0>2}.{:0>3}",
            duration.as_secs(),
            duration.as_millis() % 1000
        )
    } else {
        write!(
            f,
            "{:0>2}:{:0>2}.{:0>3}",
            duration.as_secs() / 60,
            duration.as_secs() % 60,
            duration.as_millis() % 1000
        )
    }
}
