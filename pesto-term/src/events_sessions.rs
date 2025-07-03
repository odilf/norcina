use color_eyre::eyre;
use pesto_core::{
    Db,
    event::{MaybeCustomEvent, Session},
};
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Style, Stylize as _, palette::tailwind::SLATE},
    text::Text,
    widgets::{Block, HighlightSpacing, List, ListState, StatefulWidget, Widget},
};

#[derive(Debug, Clone)]
pub struct EventItem {
    event: MaybeCustomEvent,
    session_list_state: ListState,
    sessions: Vec<Session>,
    scramble: Option<String>,
}

impl From<&EventItem> for Text<'_> {
    fn from(value: &EventItem) -> Self {
        Text::from(format!(
            "{}/{}",
            value.event.short_name(),
            value.sessions[0].name()
        ))
    }
}

#[derive(Debug, Clone)]
pub struct EventSessions {
    items: Vec<EventItem>,
    state: ListState,
}

impl EventSessions {
    pub fn new(db: &mut Db) -> eyre::Result<Self> {
        Ok(Self {
            items: db
                .get_events_and_sessions()?
                .into_iter()
                .map(|(event, sessions)| EventItem {
                    event,
                    sessions,
                    session_list_state: ListState::default().with_selected(Some(0)),
                    scramble: Some("R U R' U'".to_string()),
                })
                .collect(),
            state: ListState::default().with_selected(Some(1)),
        })
    }

    pub fn select_next(&mut self) {
        self.state.select_next();
    }

    pub fn select_previous(&mut self) {
        self.state.select_previous();
    }

    pub fn select_first(&mut self) {
        self.state.select_first();
    }

    pub fn select_last(&mut self) {
        self.state.select_last();
    }

    pub fn selected_event(&self) -> &MaybeCustomEvent {
        &self.items[self.state.selected().expect("Always something is selected")].event
    }
    pub fn selected_session(&self) -> &Session {
        let item = &self.items[self.state.selected().expect("Always something is selected")];
        &item.sessions[item
            .session_list_state
            .selected()
            .expect("Always something is selected")]
    }

    pub fn current_scramble(&self) -> &str {
        let item = &self.items[self.state.selected().expect("Always something is selected")];
        item.scramble
            .as_ref()
            .expect("Active session should always have a scramble present")
    }
}

impl Widget for &mut EventSessions {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(" Event/session ");

        let list = List::new(&self.items)
            .block(block)
            .highlight_style(Style::new().bg(SLATE.c800).green())
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}
