use norcina_core::types::{Axis, Direction};

use crate::mov::{Amount, CoreMove};

use super::{Face, Vertex};

/// The main pieces of a Pyraminx. There are 6 of them.
///
/// Edge positions correspond to 3x3 centers, in fact. There's six of them in
/// both, and if you inscribe a Pyraminx inside a 3x3 the Pyraminx edges exactly
/// overlap with the 3x3's centers.
///
/// # Orientation
///
/// They also have an orientation. Two of the moves toggle orientation, the
/// other two don't. This decision is arbitrary, so let's arbitrarily choose the
/// moves not on the +x axis, i.e., L and B moves toggle orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    // Packed field: ----oaad.
    //
    // d: direction
    // aa: axis
    // o: orientation
    data: u8,
}

impl Edge {
    #[inline(always)]
    pub const fn solved(axis: Axis, direction: Direction) -> Self {
        Self {
            data: direction.u8() + (axis.u8() << 2),
        }
    }

    pub const SOLVED: [Self; 6] = [
        Self::from_index(0),
        Self::from_index(1),
        Self::from_index(2),
        Self::from_index(3),
        Self::from_index(4),
        Self::from_index(5),
    ];

    #[inline(always)]
    pub const fn from_index(index: u8) -> Self {
        Self { data: index }
    }

    #[inline(always)]
    pub const fn u8(self) -> u8 {
        self.data
    }

    #[inline(always)]
    pub const fn axis_direction(self) -> Direction {
        Direction::from_u8_mod2(self.u8())
    }

    #[inline(always)]
    pub const fn axis(self) -> Axis {
        Axis::from_u8((self.u8() >> 1) & 0b11)
    }

    pub const fn is_oriented(self) -> bool {
        self.u8() >> 3 == 0
    }

    pub fn position_from_faces([f1, f2]: [Face; 2]) -> Self {
        //                     Shared
        // xor   00 01 10 11     axis 00 01 10 11
        //
        //  00   00 01 10 11     00   -- 10 01 00
        //  01   -- 00 11 10     01   -- -- 00 01
        //  10   -- -- 00 01     10   -- -- -- 10
        //  11   -- -- -- 00     11   -- -- -- --
        //
        // In summary, you xor and flip. Guess what that is: <->. Makes sense,
        // since we're checking equality. The thing that turns out particularly
        // nicely somewhat coincidentally is that we don't need to transform
        // the bitpattern any further because if the y bit matches, we get 01
        // which is the y-axis index, same for the z bit, 10 and the z-axis and
        // finally the x axis only appears if both are different, for which we
        // get 00 which is, again, the x-axis index!
        let shared_axis = Axis::from_u8_mod3(f1.vertex.u8() ^ f2.vertex.u8() ^ 0b11);

        // We flip the direction because the position of the edge is opposite to
        // the vertices that define their faces.
        let direction = f1.vertex.direction_on_axis(shared_axis).u8() ^ 0b1;
        debug_assert_eq!(
            f1.vertex.direction_on_axis(shared_axis),
            f2.vertex.direction_on_axis(shared_axis),
            "f1={f1:?}, f2={f2:?}, shared_axis={shared_axis:?}"
        );
        Self {
            data: direction + (shared_axis.u8() << 1),
        }
    }

    /// The two vertices this edge "connecting" (even though it doesn't really
    /// touch the vertex itself)
    pub fn vertices(self) -> [Vertex; 2] {
        let mut v1 = 0b00;
        let mut v2 = 0b11;

        let d = self.axis_direction().u8();
        // We xor with the direction, because, for the x-axis, if direction is
        // 0 the coordinates we want are 00 and 11, otherwise we want 01 and
        // 10. For the other axes we overwrite it so it doesn't matter. But this
        // is branchless!
        v1 ^= d;
        v2 ^= d;

        let a = self.axis().u8();

        // If the axis is not x, overwrite the data in that axis by the axis direction.
        //        clear   clear if d is 0.
        v1 = v1 & (!((1 << a) >> 1)) | ((d << a) >> 1);
        v2 = v2 & (!((1 << a) >> 1)) | ((d << a) >> 1);

        [Vertex::from_u8(v1), Vertex::from_u8(v2)]
    }

    /// Faces that make up this edge.
    ///
    /// The opposite of [`Edge::vertices`].
    pub fn faces(self) -> [Face; 2] {
        // Exact same implementation as `Edge::vertices`, except we flip the `d`.
        let mut v1 = 0b00;
        let mut v2 = 0b11;

        let d = self.axis_direction().u8() ^ 0b1;
        v1 ^= d;
        v2 ^= d;

        let a = self.axis().u8();

        v1 = v1 & (!((1 << a) >> 1)) | ((d << a) >> 1);
        v2 = v2 & (!((1 << a) >> 1)) | ((d << a) >> 1);

        [
            Face::new(Vertex::from_u8(v1)),
            Face::new(Vertex::from_u8(v2)),
        ]
    }

    /// Either the U face or the face in the clockwise U direction.
    pub fn orientation_face(self) -> Face {
        // An edge has a U face if the axis direction is negative
        if self.axis_direction() == Direction::Negative {
            Face::U
        } else {
            // Otherwise, depends
            // Axis:      Numerically:
            // X -> B  |  0 -> 2
            // Y -> L  |  1 -> 1
            // Z -> R  |  2 -> 3
            //
            // (4 - axis) % 3 + 1. Long story
            Face {
                vertex: Vertex::from_u8((4 - self.axis().u8()) % 3 + 1),
            }
        }
    }

    pub fn non_orientation_face(self) -> Face {
        // Similar to `Self::orientation_face`, negative => has a U face
        if self.axis_direction() == Direction::Negative {
            // This time, the non-U face depends on the axis too.
            // Namely:    And numerically:
            // X -> R     0 -> 3
            // Y -> B     1 -> 2
            // Z -> L     2 -> 1
            //
            // 3 - axis
            Face {
                vertex: Vertex::from_u8(3 - self.axis().u8()),
            }
        } else {
            // Also depends,
            // axis:       Numerically:
            // X -> L      00 -> 01
            // Y -> R      01 -> 11
            // Z -> B      10 -> 10
            //
            // Swap bits + 1
            let a = self.axis().u8();
            let swapped = ((a >> 1) | (a << 1)) & 0b11;
            Face {
                vertex: Vertex::from_u8(swapped + 1),
            }
        }
    }

    /// Whether the piece is on the U face.
    pub fn is_on_orientation_face(self) -> bool {
        self.axis_direction() == Direction::Negative
    }
}

pub fn move_pieces(mut edges: [Edge; 6], mov: CoreMove) -> [Edge; 6] {
    move_pieces_in_place(&mut edges, mov);
    edges
}

pub fn move_pieces_in_place(edges: &mut [Edge; 6], mov: CoreMove) {
    // TODO: Maybe we could have `mov.vertex_axis()` functions to be more
    // efficient with unpacking and stuff.
    let [a, b, c] = [
        (mov.vertex().x().u8() + (Axis::X.u8() << 1)) as usize,
        (mov.vertex().y().u8() + (Axis::Y.u8() << 1)) as usize,
        (mov.vertex().z().u8() + (Axis::Z.u8() << 1)) as usize,
    ];

    // TODO: Try to do branchless
    if mov.amount() == Amount::Reverse {
        // TODO: Probably can be more efficient.
        edges[a].data ^= (Edge::from_index(a as u8).is_on_orientation_face() as u8) << 3;
        edges[b].data ^= (Edge::from_index(b as u8).is_on_orientation_face() as u8) << 3;
        edges[c].data ^= (Edge::from_index(c as u8).is_on_orientation_face() as u8) << 3;
    }

    // TODO: Would be nice to remove branching.
    if mov.amount() == Amount::Single {
        edges.swap(a, b);
        edges.swap(b, c);
    } else {
        edges.swap(b, c);
        edges.swap(a, b);
    }

    // Toggle orientations if the piece moves into the U face.
    // TODO: Try to do branchless
    if mov.amount() == Amount::Single {
        // TODO: Probably can be more efficient.
        edges[a].data ^= (Edge::from_index(a as u8).is_on_orientation_face() as u8) << 3;
        edges[b].data ^= (Edge::from_index(b as u8).is_on_orientation_face() as u8) << 3;
        edges[c].data ^= (Edge::from_index(c as u8).is_on_orientation_face() as u8) << 3;
    }
}
