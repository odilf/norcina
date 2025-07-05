mod center;
pub mod edge;
mod tip;

use std::fmt;

pub use center::Centers;
pub use edge::Edge;
use norcina_core::types::{Axis, Direction, Orientation3};
pub use tip::Tips;

/// One of the 4 "vertices" of the Pyraminx, which give the names to the moves.
///
/// # Invariants
/// Bitpattern is always either 0, 1, 2 or 3.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Vertex {
    // Packed field: ------yx
    data: u8,
}

impl Vertex {
    #[inline(always)]
    pub const fn u8(self) -> u8 {
        self.data
    }

    #[inline(always)]
    pub const fn from_u8(data: u8) -> Self {
        assert!(data < 4);
        Self { data }
    }

    #[inline(always)]
    pub const fn from_u8_mod4(data: u8) -> Self {
        Self { data: data & 0b11 }
    }

    #[inline(always)]
    fn from_i8_mod4(n: i8) -> Vertex {
        Self {
            data: n.rem_euclid(4) as u8,
        }
    }

    pub const ALL: [Self; 4] = [Self::U, Self::L, Self::B, Self::R];

    //                                 zy
    pub const U: Self = Self { data: 0b00 };
    pub const L: Self = Self { data: 0b01 };
    pub const B: Self = Self { data: 0b10 };
    pub const R: Self = Self { data: 0b11 };

    #[inline(always)]
    pub const fn x(self) -> Direction {
        Direction::from_u8_any((self.data ^ (self.data >> 1)) & 0b1)
    }

    #[inline(always)]
    pub const fn y(self) -> Direction {
        Direction::from_u8_mod2(self.data)
    }

    #[inline(always)]
    pub const fn z(self) -> Direction {
        Direction::from_u8_any(self.data & 0b10)
    }

    #[inline(always)]
    pub const fn direction_on_axis(self, axis: Axis) -> Direction {
        //        -------zy-------    -----------------x---------------
        let zyx = (self.data << 1) ^ (self.data & 0b1) ^ (self.data >> 1);
        Direction::from_u8_any(zyx & (1 << axis.u8()))
    }

    /// Whether the piece is oriented when the piece is looking towards the move
    /// corresponding with this vertex, or towards the opposite side. In other
    /// words, positive for R and U, negative for L and B.
    ///
    /// See also [`Edge`](Edge#orientation)
    #[inline(always)]
    pub fn orientation_direction(self) -> Direction {
        self.x()
    }

    /// Finds the vertex that is the desired orientation offseted from `start`
    /// around `self`. That is, the vertex that is `orientation` clockwise turns
    /// away from `start` around `self`.
    ///
    /// For example,
    /// - `offset(R, U, 0) == U`
    /// - `offset(R, U, 1) == B`
    /// - `offset(R, U, 2) == L`
    ///
    /// To spell it out, all cycles are:
    ///
    /// ```text
    /// Base |       Cycle
    ///   R  | [L -> U -> B -> L]
    ///   L  | [U -> R -> B -> U]
    ///   B  | [R -> U -> L -> R]
    ///   U  | [R -> L -> B -> R]
    /// ```
    ///
    /// # Panics
    ///
    /// If `self == start`
    #[inline(always)]
    pub fn offset(self, start: Vertex, orientation: Orientation3) -> Self {
        let anchor = self.u8();
        let start = start.u8();
        assert_ne!(anchor, start);

        // If you inspect the table, you see that if the first bit is 0 you need
        // to subtract one to the vertex, otherwise you add one.
        let dont_negate = (anchor as i8 ^ 0b01) & 0b01;
        let sign = dont_negate ^ (dont_negate - 1);

        // From https://graphics.stanford.edu/~seander/bithacks.html#ConditionalNegate
        // Subtract if 0, add if 1.
        let diff = sign * orientation.u8() as i8;

        // In principle, `output = start + diff`. However, we want to skip `anchor` as a value if we were to hit it.
        // We first move to a space where the anchor is always 3. Then we can do
        // mod3 to avoid the anchor. Then we move back to the normal space.

        // m3s = mod 3 space
        let start_m3s = (start + (3 - anchor)) % 4;
        // Diff is relative, rotation of space doesn't affect it.
        let end_m3s = (start_m3s as i8 + diff).rem_euclid(3);
        let end = end_m3s - (3 - anchor as i8);

        Vertex::from_i8_mod4(end)

        // NOTE: An alternative approach, since orientation is either 0b00,
        // 0b01 or 0b10, is to switch the first two bits of the orientation,
        // but doing it conditionally is a bit trickier, probably not even that
        // different.
    }

    /// Offsets a face in the same way as [`Self::offset`].
    ///
    /// Although semantics are kind of differnet, the implementation is
    /// literally just a [`Self::offset`] wrapper.
    pub fn offset_face(self, face: Face, orientation: Orientation3) -> Face {
        Face {
            vertex: self.offset(face.vertex, orientation),
        }
    }
}

impl fmt::Debug for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Vertex {{ {} }}",
            match *self {
                Self::R => 'R',
                Self::L => 'L',
                Self::B => 'B',
                Self::U => 'U',
                _ => unreachable!(),
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Face {
    // The vertex opposite to the given face.
    pub vertex: Vertex,
}

pub type Sticker = Face;

impl Face {
    pub const R: Face = Face { vertex: Vertex::R };
    pub const L: Face = Face { vertex: Vertex::L };
    pub const B: Face = Face { vertex: Vertex::B };
    pub const U: Face = Face { vertex: Vertex::U };

    pub const fn new(opposite_vertex: Vertex) -> Self {
        Self {
            vertex: opposite_vertex,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fn_vertex_x_logic_is_correct() {
        for (input, output) in [(0b00, 0), (0b01, 1), (0b10, 1), (0b11, 0)] {
            assert_eq!(Vertex { data: input }.x().u8(), output)
        }
    }

    #[test]
    fn fn_offset_has_correct_results_for_every_input() {
        let r = Vertex::R;
        let l = Vertex::L;
        let u = Vertex::U;
        let b = Vertex::B;

        for (anchor, cycle) in [
            (u, [r, l, b]),
            (r, [l, u, b]),
            (l, [u, r, b]),
            (b, [r, u, l]),
        ] {
            for start in 0..3 {
                for i in 0..3 {
                    let orientation = Orientation3::from_u8(i as u8);
                    assert_eq!(
                        anchor.offset(cycle[start], orientation),
                        cycle[(start + i) % 3],
                        "Expected anchor {anchor:?} with face {2:?} offseted by {orientation:?} to result in {1:?} (from cycle {cycle:?}), but it resulted in {0:?}",
                        anchor.offset(cycle[start], orientation),
                        cycle[(start + i) % 3],
                        cycle[start],
                    );
                }
            }
        }
    }

    #[test]
    fn rotating_either_of_a_pair_of_vertices_around_the_other_results_in_the_same_vertex() {
        for v1 in [Vertex::R, Vertex::L, Vertex::U, Vertex::B] {
            for v2 in [Vertex::R, Vertex::L, Vertex::U, Vertex::B] {
                if v1 == v2 {
                    continue;
                }

                for offset in [Orientation3::ONE, Orientation3::TWO] {
                    assert_eq!(v1.offset(v2, offset), v2.offset(v1, -offset))
                }
            }
        }
    }
}
