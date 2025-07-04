use std::fmt;

use enum_dispatch::enum_dispatch;

pub use norcina_core::*;
pub use norcina_cube_n as cube_n;
pub use norcina_cube3 as cube3;

#[enum_dispatch(Move, MoveDisplay)]
pub enum DynMove {
    Cube3(cube3::Move),
}

impl fmt::Display for DynMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cube3(mov) => fmt::Display::fmt(mov, f),
        }
    }
}

/// Generate scrambles according to the [WCA regulations]
///
/// [WCA regulations]: https://www.worldcubeassociation.org/regulations/#4b
///
/// Specifically, "sufficiently many random moves" are interpreted, for each
/// corresponding puzzle, as:
/// - 5x5x5 Cube: 60
/// - 6x6x6 Cube: 80
/// - 7x7x7 Cube: 100
/// - Megaminx: 77
///
/// This follows [csTimer's](https://cstimer.net) conventions.
pub fn gen_scramble(event: Event, _rng: &mut impl rand::Rng) -> Alg<DynMove> {
    match event {
        Event::Cube3 => Alg {
            moves: cube3::algs::pll::T
                .into_iter()
                .map(DynMove::Cube3)
                .collect(),
        },
        _ => Alg { moves: Vec::new() },
    }
}
