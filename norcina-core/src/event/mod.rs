use core::fmt;

/// Official WCA events.
///
/// As shown here: <https://www.worldcubeassociation.org/results/records>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Event {
    Cube2,
    #[default]
    Cube3,
    Cube4,
    Cube5,
    Cube6,
    Cube7,
    Blind3,
    Blind4,
    Blind5,
    Multiblind,
    FewestMoves,
    OneHanded,
    Clock,
    Megaminx,
    Pyraminx,
    Skewb,
    Square1,
}

impl Event {
    /// The WCA id of the event.
    ///
    /// As used in "www.worldcubeassociation.org/results/records?event_id={id}" <- here
    pub fn str_id(self) -> &'static str {
        match self {
            Self::Cube2 => "222",
            Self::Cube3 => "333",
            Self::Cube4 => "444",
            Self::Cube5 => "555",
            Self::Cube6 => "666",
            Self::Cube7 => "777",
            Self::Blind3 => "333bf",
            Self::Blind4 => "444bf",
            Self::Blind5 => "555bf",
            Self::Multiblind => "333mbf",
            Self::FewestMoves => "333fm",
            Self::OneHanded => "333oh",
            Self::Clock => "clock",
            Self::Megaminx => "minx",
            Self::Pyraminx => "pyram",
            Self::Skewb => "skewb",
            Self::Square1 => "sq-1",
        }
    }

    /// The full name of the event, as used in the WCA.
    pub fn full_name(self) -> &'static str {
        match self {
            Self::Cube2 => "2x2x2 Cube",
            Self::Cube3 => "3x3x3 Cube",
            Self::Cube4 => "4x4x4 Cube",
            Self::Cube5 => "5x5x5 Cube",
            Self::Cube6 => "6x6x6 Cube",
            Self::Cube7 => "7x7x7 Cube",
            Self::Blind3 => "3x3x3 Blindfolded",
            Self::Blind4 => "4x4x4 Blindfolded",
            Self::Blind5 => "5x5x5 Blindfolded",
            Self::Multiblind => "3x3x3 Multi-Blind",
            Self::FewestMoves => "3x3x3 Fewest Moves",
            Self::OneHanded => "3x3x3 One-Handed",
            Self::Clock => "Clock",
            Self::Megaminx => "Megaminx",
            Self::Pyraminx => "Pyraminx",
            Self::Skewb => "Skewb",
            Self::Square1 => "Square-1",
        }
    }

    /// A short, very commonly used name for the event.
    ///
    /// Not an official WCA name.
    pub fn short_name(self) -> &'static str {
        match self {
            Self::Cube2 => "2x2",
            Self::Cube3 => "3x3",
            Self::Cube4 => "4x4",
            Self::Cube5 => "5x5",
            Self::Cube6 => "6x6",
            Self::Cube7 => "7x7",
            Self::Blind3 => "3BLD",
            Self::Blind4 => "4BLD",
            Self::Blind5 => "5BLD",
            Self::Multiblind => "Multiblind",
            Self::FewestMoves => "FM",
            Self::OneHanded => "OH",
            Self::Clock => "Clock",
            Self::Megaminx => "Megaminx",
            Self::Pyraminx => "Pyraminx",
            Self::Skewb => "Skewb",
            Self::Square1 => "Square-1",
        }
    }

    pub const ALL: [Event; 17] = [
        Self::Cube2,
        Self::Cube3,
        Self::Cube4,
        Self::Cube5,
        Self::Cube6,
        Self::Cube7,
        Self::Blind3,
        Self::Blind4,
        Self::Blind5,
        Self::Multiblind,
        Self::FewestMoves,
        Self::OneHanded,
        Self::Clock,
        Self::Megaminx,
        Self::Pyraminx,
        Self::Skewb,
        Self::Square1,
    ];

    pub const fn id(self) -> u8 {
        match self {
            Self::Cube2 => 0,
            Self::Cube3 => 1,
            Self::Cube4 => 2,
            Self::Cube5 => 3,
            Self::Cube6 => 4,
            Self::Cube7 => 5,
            Self::Blind3 => 6,
            Self::Blind4 => 7,
            Self::Blind5 => 8,
            Self::Multiblind => 9,
            Self::FewestMoves => 10,
            Self::OneHanded => 11,
            Self::Clock => 12,
            Self::Megaminx => 13,
            Self::Pyraminx => 14,
            Self::Skewb => 15,
            Self::Square1 => 16,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.short_name())
    }
}
