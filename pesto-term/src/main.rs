use color_eyre::eyre::{self, Context};
use crossterm::event::{self, Event, KeyCode, KeyEvent, poll};
use device_query::{DeviceQuery as _, DeviceState, Keycode};
use pesto_core::{Db, solve::Solve};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph, Widget, Wrap},
};
use std::time::{Duration, Instant, SystemTime};

mod timer;
use timer::Timer;

mod events_sessions;
use events_sessions::EventSessions;

mod solve_list;
use solve_list::SolveList;

fn main() -> eyre::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new()
        .wrap_err("Couldn't initialize app")?
        .run(&mut terminal)
        .wrap_err("Problem when running app");
    ratatui::restore();
    app_result
}

#[derive(Debug)]
pub struct App {
    timer: Timer,
    events_sessions: EventSessions,
    solve_list: SolveList,
    db: Db,
    exit: bool,
    last_draw: Instant,
    // TODO: Move this and below to config
    min_press_duration: Duration,
    min_stop_duration: Duration,
    timer_refresh_duration: Duration,
    debug: bool,
}

impl App {
    pub fn new() -> eyre::Result<App> {
        let mut db = Db::new()?;
        let events_sessions = EventSessions::new(&mut db)?;
        let solve_list = SolveList::new(
            &mut db,
            events_sessions.selected_event(),
            events_sessions.selected_session(),
        )
        .wrap_err("Couldn't get solves")?;

        Ok(App {
            timer: Timer::Idle,
            events_sessions,
            solve_list,
            db,
            exit: false,
            last_draw: Instant::now(),
            debug: false,
            min_press_duration: Duration::from_millis(100),
            min_stop_duration: Duration::from_millis(500),
            timer_refresh_duration: Duration::from_millis(16),
        })
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        while !self.exit {
            let now = Instant::now();

            if self.timer.is_running() {
                // Only redraw if 10ms have passed since last draw
                if now.duration_since(self.last_draw) >= self.timer_refresh_duration {
                    terminal.draw(|frame| self.draw(frame))?;
                    self.last_draw = now;
                }

                if poll(self.timer_refresh_duration)? {
                    self.handle_events()?;
                }
            } else {
                // Always redraw when timer isn't running (for responsiveness)
                terminal.draw(|frame| self.draw(frame))?;

                // Blocking mode: wait for events indefinitely when timer isn't running
                self.handle_events()?;
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> eyre::Result<()> {
        if self.timer.is_pressed() {
            // Busy loop waiting for release, to be more accurate
            loop {
                let device_state = DeviceState::new();
                let keys = device_state.get_keys();
                if !keys.contains(&Keycode::Space) {
                    break;
                }
            }

            self.timer.release(self.min_press_duration);

            // Discard crossterm events
            while event::poll(Duration::from_millis(0))? {
                let _ = event::read()?;
            }
            return Ok(());
        }

        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind.is_press() => {
                self.handle_keypress_event(key_event)?
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_keypress_event(&mut self, key_event: KeyEvent) -> eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::F(1) => self.debug = !self.debug,
            KeyCode::Char(' ') => {
                if let Some(time) = self.timer.press(self.min_stop_duration) {
                    self.db.insert_solve(
                        Solve::new(time, self.events_sessions.current_scramble().to_string()),
                        self.events_sessions.selected_event(),
                        self.events_sessions.selected_session(),
                    )?;

                    self.solve_list.refresh(
                        &mut self.db,
                        self.events_sessions.selected_event(),
                        self.events_sessions.selected_session(),
                    )?;
                }
            }

            // Event navigation
            KeyCode::Char('j') | KeyCode::Down => self.events_sessions.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.events_sessions.select_previous(),
            KeyCode::Char('h') | KeyCode::Left | KeyCode::Home => {
                self.events_sessions.select_first()
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter | KeyCode::End => {
                self.events_sessions.select_last();
            }
            _ => {}
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = if self.debug {
            let [debug_area, normal_area] =
                Layout::horizontal([Constraint::Length(40), Constraint::Fill(1)]).areas(area);

            Paragraph::new(format!("{self:#?}"))
                .wrap(Wrap { trim: false })
                .render(debug_area, buf);
            normal_area
        } else {
            area
        };

        let title = Line::from(" pesto-term! ".bold().green());
        let instructions = Line::from(vec![
            " start/stop timer ".white(),
            "<space>".blue().bold(),
            " select event ".white(),
            "<e>".blue().bold(),
            " quit ".white(),
            "<q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered());

        let layout = Layout::horizontal([
            Constraint::Length(30),
            Constraint::Fill(1),
            Constraint::Length(30),
        ])
        .spacing(0);

        let [left_rect, center_rect, right_rect] = layout.areas(area);

        block.render(center_rect, buf);
        self.events_sessions.render(left_rect, buf);
        self.solve_list.render(right_rect, buf);

        let layout_center = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .margin(1);

        let [scramble_rect, timer_rect, extra_rect] = layout_center.areas(center_rect);

        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                "Scramble: ".bold(),
                self.events_sessions.current_scramble().not_bold(),
            ]),
        ])
        .centered()
        .render(scramble_rect, buf);

        self.timer.render(
            timer_rect,
            buf,
            self.min_press_duration,
            self.min_stop_duration,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn render() -> eyre::Result<()> {
        let mut app = App::new()?;
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        assert_eq!(buf, expected);

        Ok(())
    }
}
