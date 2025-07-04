use std::fmt::{self, Write as _};

use norcina_cube_n::math::Face;
use norcina_cube_n::mov::Move;
use norcina_cube_n::piece::{
    corner::{self, Corner, CornerPosition},
    edge::{self, Edge, EdgePosition},
};
use owo_colors::{OwoColorize, Rgb};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cube {
    // TODO: Is it a problem if these are pub? It might be...
    pub corners: [Corner; 8],
    pub edges: [Edge; 12],
}

impl Cube {
    pub const SOLVED: Self = Cube {
        corners: Corner::SOLVED,
        edges: Edge::SOLVED,
    };

    pub fn random_with_rng(rng: &mut impl rand::Rng) -> Self {
        let mut corners = Corner::random(rng);
        let mut edges = Edge::random(rng);

        let corner_swap_parity = Corner::count_swaps(corners) % 2;
        let edge_swap_parity = Edge::count_swaps(edges) % 2;

        dbg!(corner_swap_parity, edge_swap_parity);

        if corner_swap_parity != edge_swap_parity {
            // TODO: Could we always swap the same two arbtirary pieces, or
            // would that stop it from being uniform distribution?
            if rng.random() {
                let i = rng.random_range(0..8);
                let mut j = rng.random_range(0..7);
                if j >= i {
                    j += 1;
                }

                corners.swap(i, j);
            } else {
                let i = rng.random_range(0..12);
                let mut j = rng.random_range(0..11);
                if j >= i {
                    j += 1;
                }

                edges.swap(i, j);
            }
        }

        Cube { corners, edges }
    }

    pub fn random() -> Self {
        Self::random_with_rng(&mut rand::rng())
    }

    fn sticker_at(self, face: Face, up: Face, col: i32, row: i32) -> Sticker {
        // Center sticker
        if col == 1 && row == 1 {
            return face;
        }

        let side = up.cross(face);

        if (col + row) % 2 == 0 {
            let faces = [
                face,
                if row == 0 { up } else { up.opposite() },
                if col == 0 { side.opposite() } else { side },
            ];

            let position = CornerPosition::from_faces(faces);
            let piece = position.pick(self.corners);
            corner::sticker(piece, position, face)
        } else {
            let other_face = match (row, col) {
                (0, 1) => up,
                (1, 0) => side.opposite(),
                (1, 2) => side,
                (2, 1) => up.opposite(),
                _ => unreachable!(),
            };

            let position = EdgePosition::from_faces([face, other_face]);
            let piece = position.pick(&self.edges);
            edge::sticker(piece, position, face)
        }
    }

    pub fn mov_single(self, mov: Move) -> Self {
        Self {
            corners: corner::move_pieces(self.corners, mov),
            edges: edge::move_pieces(self.edges, mov),
        }
    }

    pub fn mov(mut self, alg: impl IntoIterator<Item = Move>) -> Self {
        for mov in alg {
            self = self.mov_single(mov);
        }
        self
    }

    pub const fn is_solved(self) -> bool {
        matches!(self, Self::SOLVED)
    }

    /// An iterator of tuples of every state that is reachable from this state, and the number of moves it takes to reach it.
    pub fn neighbors(self) -> impl Iterator<Item = (Move, Self)> {
        Move::iter().map(move |mov| (mov, self.mov_single(mov)))
    }

    pub fn corners(self) -> impl Iterator<Item = (CornerPosition, Corner)> {
        self.corners.into_iter().enumerate().map(|(i, state)| {
            (
                unsafe { CornerPosition::from_index_unchecked(i as u8) },
                state,
            )
        })
    }

    pub fn edges(self) -> impl Iterator<Item = (EdgePosition, Edge)> {
        self.edges.into_iter().enumerate().map(|(i, state)| {
            (
                unsafe { EdgePosition::from_index_unchecked(i as u8) },
                state,
            )
        })
    }
}

pub type Sticker = Face;
pub type ColorScheme = fn(Face) -> Rgb;

const DEFAULT_COLOR_SCHEME: ColorScheme = |face| match face {
    Face::R => Rgb(217, 39, 39),
    Face::U => Rgb(250, 250, 250),
    Face::F => Rgb(109, 242, 116),
    Face::L => Rgb(255, 153, 12),
    Face::D => Rgb(255, 224, 0),
    Face::B => Rgb(79, 123, 212),
};

impl fmt::Debug for Cube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct CornersDebug([Corner; 8]);
        impl fmt::Debug for CornersDebug {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut dl = f.debug_list();
                for (i, corner) in self.0.into_iter().enumerate() {
                    let position = CornerPosition::from_index(i as u8);
                    dl.entry(&format_args!("{corner} is at {position}"));
                }

                dl.finish()
            }
        }

        struct EdgesDebug([Edge; 12]);
        impl fmt::Debug for EdgesDebug {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut dl = f.debug_list();
                for (i, edge) in self.0.into_iter().enumerate() {
                    let position = EdgePosition::from_index(i as u8);
                    dl.entry(&format_args!("{edge} is at {position}"));
                }

                dl.finish()
            }
        }

        f.debug_struct("Cube")
            .field("corners", &CornersDebug(self.corners))
            .field("edges", &EdgesDebug(self.edges))
            .finish()
    }
}

impl fmt::Display for Cube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let write = |f: &mut fmt::Formatter<'_>, sticker| {
            write!(f, "{}", "██".color((DEFAULT_COLOR_SCHEME)(sticker)))
        };

        let pad = |f: &mut fmt::Formatter<'_>| f.write_str("      ");

        for b_row in 0..3 {
            pad(f)?;
            for b_col in 0..3 {
                write(f, self.sticker_at(Face::B, Face::D, b_col, b_row))?;
            }
            f.write_char('\n')?;
        }
        for u_row in 0..3 {
            pad(f)?;
            for u_col in 0..3 {
                write(f, self.sticker_at(Face::U, Face::B, u_col, u_row))?;
            }
            f.write_char('\n')?;
        }

        for lfr_row in 0..3 {
            for l_col in 0..3 {
                write(f, self.sticker_at(Face::L, Face::U, l_col, lfr_row))?;
            }
            for f_col in 0..3 {
                write(f, self.sticker_at(Face::F, Face::U, f_col, lfr_row))?;
            }
            for r_col in 0..3 {
                write(f, self.sticker_at(Face::R, Face::U, r_col, lfr_row))?;
            }

            f.write_char('\n')?;
        }

        for d_row in 0..3 {
            pad(f)?;
            for d_col in 0..3 {
                write(f, self.sticker_at(Face::D, Face::F, d_col, d_row))?;
            }
            f.write_char('\n')?;
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "quickcheck"))]
mod tests {
    use quickcheck::{Arbitrary, Gen};

    use super::*;

    impl Arbitrary for Cube {
        fn arbitrary(g: &mut Gen) -> Self {
            let scramble = <Vec<Move>>::arbitrary(g);
            Cube::SOLVED.mov(scramble)
        }
    }

    #[test]
    fn debug_cube_insta() {
        insta::assert_debug_snapshot!(Cube::SOLVED)
    }

    #[test]
    fn display_cube_insta() {
        insta::assert_snapshot!(Cube::SOLVED)
    }
}
