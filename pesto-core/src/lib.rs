use norcina::Event;

pub enum CustomEvent {
    Official(Event),
    Unofficial {
        name: String,
        scramble: Option<Event>,
    },
}
