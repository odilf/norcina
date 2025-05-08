use std::fmt::{self, Write as _};

use owo_colors::{OwoColorize, Rgb};

use crate::{
    corner::{self, Corner, CornerPosition},
    edge::{self, Edge, EdgePosition},
    math::Face,
    mov::Move,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cube {
    corners: [Corner; 8],
    edges: [Edge; 12],
}

impl Cube {
    pub const SOLVED: Self = Cube {
        corners: [
            Corner::solved(0),
            Corner::solved(1),
            Corner::solved(2),
            Corner::solved(3),
            Corner::solved(4),
            Corner::solved(5),
            Corner::solved(6),
            Corner::solved(7),
        ],
        edges: [
            Edge::solved(0),
            Edge::solved(1),
            Edge::solved(2),
            Edge::solved(3),
            Edge::solved(4),
            Edge::solved(5),
            Edge::solved(6),
            Edge::solved(7),
            Edge::solved(8),
            Edge::solved(9),
            Edge::solved(10),
            Edge::solved(11),
        ],
    };

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

    pub fn mov(self, mov: Move) -> Self {
        Self {
            corners: corner::move_pieces(self.corners, mov),
            edges: edge::move_pieces(self.edges, mov),
        }
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
        for b_row in 0..3 {
            pad(f)?;
            for b_col in 0..3 {
                write(f, self.sticker_at(Face::B, Face::D, b_col, b_row))?;
            }
            f.write_char('\n')?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{quickcheck, Arbitrary, Gen};

    use super::*;

    impl Arbitrary for Cube {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                corners: [Corner::arbitrary(g); 8],
                edges: [Edge::arbitrary(g); 12],
            }
        }
    }
}
