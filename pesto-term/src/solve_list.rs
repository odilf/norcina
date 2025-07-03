use color_eyre::eyre;
use pesto_core::{
    Db,
    event::{MaybeCustomEvent, Session},
};
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Style, Stylize as _},
    text::Text,
    widgets::{Block, List, ListState, Padding, StatefulWidget, Widget},
};

#[derive(Debug, Clone)]
pub struct Solve(pesto_core::Solve);

#[derive(Debug, Clone)]
pub struct SolveList {
    solves: Vec<Solve>,
    list_state: ListState,
}

impl SolveList {
    pub fn new(db: &mut Db, event: &MaybeCustomEvent, session: &Session) -> eyre::Result<Self> {
        Ok(Self {
            solves: db
                .get_solves(event, session)?
                .into_iter()
                .rev()
                .map(Solve)
                .collect(),
            list_state: ListState::default(),
        })
    }

    pub fn refresh(
        &mut self,
        db: &mut Db,
        event: &MaybeCustomEvent,
        session: &Session,
    ) -> eyre::Result<()> {
        self.solves = db
            .get_solves(event, session)?
            .into_iter()
            .rev()
            .map(Solve)
            .collect();

        Ok(())
    }
}

impl Widget for &mut SolveList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let list = List::new(&self.solves)
            .block(
                Block::bordered()
                    .title(" Solves ")
                    .padding(Padding::horizontal(2)),
            )
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);

        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}

impl From<&Solve> for Text<'_> {
    fn from(value: &Solve) -> Self {
        Text::from(value.0.to_string())
    }
}
