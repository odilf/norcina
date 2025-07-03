use norcina::Event;

#[derive(Debug, Clone)]
pub enum Session {
    Main,
    Custom { name: String, id: usize },
}

impl Session {
    pub const fn name(&self) -> &str {
        match self {
            Self::Main => "main",
            Self::Custom { name, .. } => name.as_str(),
        }
    }

    pub const fn id(&self) -> usize {
        match self {
            Self::Main => 0,
            &Self::Custom { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CustomEvent {
    pub id: usize,
    pub name: String,
    pub scramble_type: Option<Event>,
}

#[derive(Debug, Clone)]
pub enum MaybeCustomEvent {
    Official(Event),
    Unofficial(CustomEvent),
}

impl MaybeCustomEvent {
    pub const fn scramble_type(&self) -> Option<Event> {
        match self {
            Self::Official(event) => Some(*event),
            Self::Unofficial(event) => event.scramble_type,
        }
    }

    pub fn short_name(&self) -> &str {
        match self {
            Self::Official(event) => event.short_name(),
            Self::Unofficial(event) => &event.name,
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Self::Official(event) => event.id() as usize,
            Self::Unofficial(event) => event.id,
        }
    }
}

impl Default for MaybeCustomEvent {
    fn default() -> Self {
        Self::Official(Event::default())
    }
}

pub struct EventSessionList {
    custom_events: Vec<CustomEvent>,
    sessions: Vec<Vec<Session>>,
}

impl EventSessionList {
    pub fn events(&self) -> impl Iterator<Item = MaybeCustomEvent> {
        Event::ALL
            .into_iter()
            .map(MaybeCustomEvent::Official)
            .chain(
                self.custom_events
                    .iter()
                    .map(|event| MaybeCustomEvent::Unofficial(event.clone())), // TODO: Can we remove the clone?
            )
    }

    pub fn sessions_of(&self, event: MaybeCustomEvent) -> impl Iterator<Item = &Session> {
        self.sessions[event.id()].iter()
    }
}
