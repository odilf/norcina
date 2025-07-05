use mov::{CoreMove, Move};
use norcina_core::types::Orientation3;
use owo_colors::{OwoColorize as _, Rgb};
use piece::{Centers, Edge, Face, Tips, Vertex, edge};
use std::fmt;

pub mod mov;
pub mod piece;

/// A [Pyraminx](https://www.worldcubeassociation.org/results/rankings/pyram/)
///
/// See also [`CorePyraminx`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pyraminx {
    core: CorePyraminx,
    tips: Tips,
}

impl Pyraminx {
    pub const SOLVED: Self = Self {
        core: CorePyraminx::SOLVED,
        tips: Tips::SOLVED,
    };

    pub fn mov(mut self, moves: impl IntoIterator<Item = Move>) -> Self {
        for mov in moves {
            self = self.mov_single(mov)
        }
        self
    }

    pub fn mov_single(self, mov: Move) -> Self {
        if mov.is_tip_move() {
            Self {
                tips: self.tips.mov(mov.core()),
                ..self
            }
        } else {
            Self {
                core: self.core.mov(mov.core()),
                tips: self.tips.mov(mov.core()),
            }
        }
    }

    pub fn write(
        self,
        f: &mut fmt::Formatter<'_>,
        color_scheme: ColorScheme,
        render_as_triangles: bool,
    ) -> fmt::Result {
        let mut i = 0;
        let mut write = |f: &mut fmt::Formatter<'_>, sticker| {
            if render_as_triangles {
                i += 1;
                if i % 2 != 0 {
                    write!(f, "{}", "▲".color((color_scheme)(sticker)))
                } else {
                    write!(f, "{}", "▼".color((color_scheme)(sticker)))
                }
            } else {
                write!(f, "{}", "██".color((color_scheme)(sticker)))
            }
        };

        let space = |f: &mut fmt::Formatter<'_>| {
            if render_as_triangles {
                write!(f, "{}", " ")
            } else {
                write!(f, "{}", "  ")
            }
        };

        let smallspace = |f: &mut fmt::Formatter<'_>| {
            if render_as_triangles {
                write!(f, "{}", " ")
            } else {
                write!(f, "{}", "  ")
            }
        };

        // RLU is the front face.

        for row in 0..3 {
            for _ in 0..row {
                space(f)?;
            }
            let row_rev = 2 - row;

            for index in 0..=2 * row_rev {
                write(f, self.sticker_at(Face::R, Vertex::L, row_rev, index))?;
            }

            smallspace(f)?;
            for index in (0..=2 * row).rev() {
                write(f, self.sticker_at(Face::B, Vertex::U, row, index))?;
            }

            smallspace(f)?;
            for index in 0..=2 * row_rev {
                write(f, self.sticker_at(Face::L, Vertex::R, row_rev, index))?;
            }

            writeln!(f)?;
        }

        for row in (0..3).rev() {
            for _ in 0..(6 - row) {
                space(f)?;
            }

            for index in 0..=2 * row {
                write(f, self.sticker_at(Face::U, Vertex::B, row, index))?;
            }

            writeln!(f)?;
        }

        Ok(())
    }

    fn sticker_at(&self, query_face: Face, query_base: Vertex, row: u8, index: u8) -> Face {
        debug_assert!(index <= row * 2);

        // Tips at: (0, 0), (2, 0) or (2, 4)
        // Centers at: (1, 1), (2, 1), (2, 3)
        // Edges at: (1, 0), (1, 2), (2, 2)
        //
        // In a square:
        // row/index  0 1 2 3 4
        //     2      T C E C T
        //     1      E C E
        //     0      T
        //
        // Center if index is odd.
        //
        // Without centers:
        // row/index  0 2 4
        //     2      T E T
        //     1      E E
        //     0      T
        //
        // Edge if either if index == 2 or row == 1

        let is_center = index % 2 != 0;
        let is_edge_or_center = index == 2 || row == 1;
        // We start off with edges, because centers and tips are basically identical.
        if is_edge_or_center && !is_center {
            // We have a face
            // B---C
            //  \ /
            //   A
            // We want to get the vertices that touch the edge.

            // We first get the "index"
            //  --.2--    0 -> (1, 0)    mapping function: f(r, i) = r + i/2 - 1
            //  \    /    1 -> (1, 2)
            //   0  1     2 -> (2, 2)
            //    \/
            let i = row + index / 2 - 1;

            // Get the "other" vertex
            let other_vertex = query_face
                .vertex
                .offset(query_base, Orientation3::from_u8_mod3(i + 1));
            let position_faces = [query_face, Face::new(other_vertex)];

            // TODO: Maybe make a proper `EdgePosition` struct...
            let edge_position = Edge::position_from_faces(position_faces);
            let edge = self.core.edges[edge_position.u8() as usize];

            if (query_face == edge_position.orientation_face()) == edge.is_oriented() {
                edge.orientation_face()
            } else {
                edge.non_orientation_face()
            }
        } else {
            let is_center = index % 2 != 0;
            let (i, set) = if is_center {
                (row.max(index) - 1, self.core.centers)
            } else {
                (row / 2 + index / 4, self.tips.as_centers())
            };

            // Find the relevant vertex.
            let vertex = query_face
                .vertex
                .offset(query_base, -Orientation3::from_u8(i));

            // Find the color by "undoing" the given rotation.
            vertex.offset_face(query_face, -set.orientation_of(vertex))
        }
    }
}

/// The parts that are "interesting" of a [`Pyraminx`]. That is, the
/// [`Pyraminx`] without the tips.
///
/// This is because solving (or manipulating in any way) the tips is trivial:
/// Every tip is independent from each other and they can be in one of 3 states
/// without affecting each other. The number of states is not thaaat low (3⁴ =
/// 81) but it's extremely easy to basically map out all states, because the
/// transition between them are, again, very straightforward.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorePyraminx {
    centers: Centers,
    edges: [Edge; 6],
}

impl CorePyraminx {
    pub const SOLVED: Self = Self {
        centers: Centers::SOLVED,
        edges: Edge::SOLVED,
    };

    pub fn mov(self, mov: CoreMove) -> Self {
        Self {
            centers: self.centers.mov(mov),
            edges: edge::move_pieces(self.edges, mov),
        }
    }
}

pub type ColorScheme = fn(Face) -> Rgb;

pub const DEFAULT_COLOR_SCHEME: ColorScheme = |sticker| match sticker {
    Face::R => Rgb(214, 59, 39),
    Face::L => Rgb(23, 125, 235),
    Face::B => Rgb(39, 214, 95),
    Face::U => Rgb(235, 231, 23),
    _ => unreachable!(),
};

impl fmt::Display for Pyraminx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f, DEFAULT_COLOR_SCHEME, false)
    }
}

#[cfg(feature = "quickcheck")]
mod quickcheck_impl {
    use quickcheck::{Arbitrary, Gen};
}

#[cfg(test)]
mod tests {
    use mov::Amount;

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn snapshot_solved_pyraminx() {
        insta::assert_snapshot!(Pyraminx::SOLVED)
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn snapshot_every_move() {
        for vertex in Vertex::ALL {
            for amount in [Amount::Single, Amount::Reverse] {
                for tip in [false, true] {
                    dbg!(vertex, amount, tip);
                    let mov = Move::new(vertex, amount, tip);
                    insta::assert_snapshot!(
                        format!("Move: {mov} (tip={tip})"),
                        Pyraminx::SOLVED.mov([mov])
                    );
                }
            }
        }
    }

    #[cfg(feature = "quickcheck")]
    quickcheck::quickcheck! {
         // fn move_and_inverse_is_identity(mov: Move, state: Pyraminx) -> bool {
         //     state.mov([mov, mov.inverse()]) == state
         // }

         // fn triple_move_is_identity(mov: Move, state: Pyraminx) -> bool {
         //     state.mov([mov, mov, mov]) == state
         // }

         // fn from_faces_faces_is_identity(f1: Face, f2: Face) -> bool {
         //     let n = Edge::position_from_faces([f1, f2]).faces();
         //     n.contains(&f1) && n.contains(&f2)
         // }
    }
}
