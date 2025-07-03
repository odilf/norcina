use std::time::{Duration, Instant};

use ratatui::{
    crossterm::style::Color,
    layout::{Constraint, Flex, Layout},
    prelude::{Buffer, Rect},
    style::Stylize as _,
    text::Line,
    widgets::Widget,
};
use tui_big_text::{BigText, PixelSize};

#[derive(Debug, Default, Clone, Copy)]
pub enum Timer {
    /// Before having pressed anything, "white"
    #[default]
    Idle,

    /// Started pressing on the timer, not released yet
    Pressed { press_start: Instant },

    /// Timer is running.
    Running { start: Instant },

    Stopped {
        time: Duration,
        stopped_instant: Instant,
    },
}

impl Timer {
    pub fn press(&mut self, min_stop_duration: Duration) -> Option<Duration> {
        match self {
            // If idle, start
            Self::Idle => {
                *self = Self::Pressed {
                    press_start: Instant::now(),
                }
            }

            // If pressed and pressed again, just keep pressing
            Self::Pressed { .. } => (),

            // If running, stop and return time.
            Self::Running { start } => {
                let time = Instant::now().duration_since(*start);
                *self = Self::Stopped {
                    time,
                    stopped_instant: Instant::now(),
                };
                return Some(time);
            }

            // If stopped, make sure enough time has passed, then press.
            Self::Stopped {
                stopped_instant, ..
            } => {
                if Instant::now().duration_since(*stopped_instant) >= min_stop_duration {
                    *self = Self::Pressed {
                        press_start: Instant::now(),
                    }
                }
            }
        };

        None
    }

    pub fn release(&mut self, min_press_duration: Duration) {
        match self {
            Self::Pressed { press_start } => {
                let press_duration = Instant::now().duration_since(*press_start);
                if press_duration < min_press_duration {
                    *self = Self::Idle
                } else {
                    *self = Self::Running {
                        // NOTE: We recalculate the start time to be more accurate.
                        start: Instant::now(),
                    }
                }
            }
            Self::Idle | Self::Running { .. } | Self::Stopped { .. } => (),
        }
    }

    pub const fn is_pressed(&self) -> bool {
        matches!(self, Self::Pressed { .. })
    }

    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        min_press_duration: Duration,
        min_stop_duration: Duration,
    ) {
        let (duration, color) = match *self {
            Timer::Idle => (Duration::ZERO, Color::White),
            Timer::Pressed { press_start } => {
                if Instant::now().duration_since(press_start) < min_press_duration {
                    (Duration::ZERO, Color::Yellow)
                } else {
                    (Duration::ZERO, Color::Green)
                }
            }
            Timer::Running { start } => (Instant::now().duration_since(start), Color::Blue),
            Timer::Stopped {
                time,
                stopped_instant,
            } => {
                if Instant::now().duration_since(stopped_instant) < min_stop_duration {
                    (time, Color::Green)
                } else {
                    (time, Color::White)
                }
            }
        };

        let mins = format!("{:0>2}:", duration.as_secs() / 60);
        let secs = format!("{:0>2}", duration.as_secs() % 60);
        let milis = format!(".{:0>3}", duration.as_millis() % 1000);

        let [mins_area, secs_area, milis_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(16),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, milis_area] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
            .flex(Flex::Start)
            .areas(milis_area);

        if duration >= Duration::from_secs(60) {
            BigText::builder()
                .right_aligned()
                .pixel_size(PixelSize::HalfHeight)
                .lines([Line::from(mins.fg(color))])
                .build()
                .render(mins_area, buf);
        }

        BigText::builder()
            .centered()
            .pixel_size(PixelSize::HalfHeight)
            .lines([Line::from(secs.fg(color))])
            .build()
            .render(secs_area, buf);

        BigText::builder()
            .left_aligned()
            .pixel_size(PixelSize::Sextant)
            .lines([Line::from(milis.fg(color))])
            .build()
            .render(milis_area, buf);
    }
}
