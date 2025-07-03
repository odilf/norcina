use std::{array, fmt, mem::transmute};

use crate::{
    Sticker,
    math::{Axis, Direction, Face},
    mov::{Amount, Move},
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    /// Packed field `---onnba`
    data: u8,
}

impl Edge {
    #[inline]
    pub const fn a(self) -> Direction {
        // TODO: Transmute
        Direction::from_bool(self.data & 0b01 != 0)
    }

    #[inline]
    pub const fn b(self) -> Direction {
        // TODO: Transmute
        Direction::from_bool(self.data & 0b10 != 0)
    }

    #[inline]
    pub const fn normal(self) -> Axis {
        // TODO: Transmute/unchecked
        Axis::from_u8((self.data >> 2) & 0b11)
    }

    #[inline]
    pub const fn orientation(self) -> Direction {
        // TODO: Transmute/unchecked
        Direction::from_u8(self.data >> 4)
    }

    pub const SOLVED: [Edge; 12] = [
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
    ];

    pub const fn solved(index: u8) -> Edge {
        assert!(index < 12);
        Edge { data: index }
    }

    #[inline]
    pub const fn is_oriented(self) -> bool {
        //            ---onnba
        self.data & 0b00010000 == 0
    }

    #[inline]
    pub fn position(self) -> EdgePosition {
        // SAFETY: Both [`Edge`] and [`EdgePosition`] are a single `u8` in memory, and we are removing the orientation bit.
        unsafe { EdgePosition::from_index_unchecked(self.data & 0b01111) }
    }

    /// Returns a possible set of 12 edges.
    ///
    /// The orientation parity is always positive/0/false.
    pub fn random(rng: &mut impl rand::Rng) -> [Edge; 12] {
        use rand::seq::SliceRandom;

        let mut out = Self::SOLVED;
        out.shuffle(rng);

        let mut final_orientation = false;
        for corner in &mut out[0..11] {
            let orientation = rng.random_bool(0.5);
            corner.data += (orientation as u8) << 3;
            final_orientation ^= orientation;
        }

        // TODO: Does this work?
        out[11].data += (final_orientation as u8) << 3;

        out
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EdgePosition {
    /// Packed field `---onnab`
    ///
    /// Same as [`Edge`]'s layout. That means the orientation bit might be set, even though it doesn't
    /// mean anything for an edge's position.
    data: u8,
}

impl EdgePosition {
    #[inline]
    pub const fn a(self) -> Direction {
        Direction::from_bool(self.data & 0b01 != 0)
    }

    #[inline]
    pub const fn b(self) -> Direction {
        Direction::from_bool(self.data & 0b10 != 0)
    }

    #[inline]
    pub const fn normal(self) -> Axis {
        Axis::from_u8((self.data >> 2) & 0b11)
    }

    pub fn from_faces([f1, f2]: [Face; 2]) -> Self {
        let normal = Axis::other(f1.axis(), f2.axis());
        let properly_ordered = f1.axis() == normal.next();
        let (a, b) = if properly_ordered {
            (f1.direction(), f2.direction())
        } else {
            (f2.direction(), f1.direction())
        };

        EdgePosition {
            #[allow(clippy::identity_op)]
            data: (a.u8() << 0) + (b.u8() << 1) + (normal.u8() << 2),
        }
    }

    pub const fn faces(self) -> [Face; 2] {
        let normal = self.normal();
        let a = Direction::from_bool(self.data & 0b01 != 0);
        let b = Direction::from_bool(self.data & 0b10 != 0);
        [Face::new(normal.next(), a), Face::new(normal.prev(), b)]
    }

    // TODO: Maybe this should take the array directly. It's just 12 bytes...
    pub fn pick(self, edges: &[Edge; 12]) -> Edge {
        edges[self.data as usize]
    }

    /// # Safety
    ///
    /// Index must be between 0 and 11.
    pub unsafe fn from_index_unchecked(index: u8) -> EdgePosition {
        debug_assert!(index < 12);
        // SAFETY: Numbers between 0 and 11 are valid edge positions.
        unsafe { transmute(index) }
    }

    #[inline]
    pub const fn from_index(index: u8) -> Self {
        assert!(index < 12);
        EdgePosition { data: index }
    }

    #[inline]
    fn direction_on_axis(self, axis: Axis) -> Direction {
        debug_assert_ne!(
            self.normal(),
            axis,
            "Tried to get the direction along normal"
        );

        let shift = (3 - self.normal().u8() + axis.u8()) % 3 - 1;
        Direction::from_u8((self.data >> shift) & 0b1)
    }

    #[inline]
    fn face_on_axis(self, axis: Axis) -> Face {
        Face::new(axis, self.direction_on_axis(axis))
    }

    #[inline]
    pub fn orientation_axis(self) -> Axis {
        // TODO: Do this with bit twiddling
        if self.normal() == Axis::Y {
            Axis::Z
        } else {
            Axis::Y
        }
    }

    pub fn non_orientation_axis(self) -> Axis {
        match self.normal() {
            Axis::X => Axis::Z,
            Axis::Y => Axis::X,
            Axis::Z => Axis::X,
        }
    }

    pub fn orientation_face(self) -> Face {
        self.face_on_axis(self.orientation_axis())
    }

    pub fn other_face(self) -> Face {
        self.face_on_axis(self.non_orientation_axis())
    }

    pub fn contains_face(self, face: Face) -> bool {
        face.axis() != self.normal() && face.direction() == self.direction_on_axis(face.axis())
    }

    /// The minimum amount of turns to get from `self` to `other`.
    ///
    /// There is
    /// - 1 position where this value is 0 (itself),
    /// - 6 positions where the value is 1 and
    /// - 4 positions where the value is 2.
    pub fn turn_distance(self, other: EdgePosition) -> u8 {
        let [f1, f2] = self.faces();
        let [g1, g2] = other.faces();

        let f1_shared = f1 == g1 || f1 == g2;
        let f2_shared = f2 == g1 || f2 == g2;
        let shared_faces = f1_shared as u8 + f2_shared as u8;
        2 - shared_faces
    }

    pub const ALL: [EdgePosition; 12] = [
        EdgePosition::from_index(0),
        EdgePosition::from_index(1),
        EdgePosition::from_index(2),
        EdgePosition::from_index(3),
        EdgePosition::from_index(4),
        EdgePosition::from_index(5),
        EdgePosition::from_index(6),
        EdgePosition::from_index(7),
        EdgePosition::from_index(8),
        EdgePosition::from_index(9),
        EdgePosition::from_index(10),
        EdgePosition::from_index(11),
    ];

    pub const fn index(self) -> u8 {
        // TODO: Maybe transmute
        self.data
    }

    pub fn with_orientation(self, orientation: Direction) -> Edge {
        Edge {
            data: self.data + (orientation.u8() << 4),
        }
    }
}

pub fn sticker(edge: Edge, position: EdgePosition, face: Face) -> Sticker {
    assert_ne!(face.axis(), position.normal());
    if (position.orientation_axis() == face.axis()) == edge.is_oriented() {
        edge.position().orientation_face()
    } else {
        edge.position().other_face()
    }
}

pub fn move_pieces(edges: [Edge; 12], mov: Move) -> [Edge; 12] {
    array::from_fn(|i| {
        let position = EdgePosition::from_index(i as u8);
        let (dir_mov, other_axis_offset) = if mov.face().axis() == position.normal().next() {
            (position.a(), 1)
        } else if mov.face().axis() == position.normal().prev() {
            (position.b(), 0)
        } else {
            return edges[i];
        };

        if dir_mov != mov.face().direction() {
            return edges[i];
        }

        assert_ne!(mov.face().axis(), position.normal());
        // Example to what happens to bits on an R move:
        //    RU    ->    RB    ->    RD    ->    RF    ->    RU
        // ---01000 -> ---00101 -> ---01010 -> ---00100 -> ---01000
        // ---onnab    ---onnab    ---onnab    ---onnab    ---onnab
        let new_edge_pos = match (mov.amount(), mov.face().direction()) {
            (Amount::Double, _) => return edges[i ^ (0b1 << other_axis_offset) as usize],
            (Amount::Single, Direction::Positive) | (Amount::Reverse, Direction::Negative) => {
                // Normal switches to the one that isn't the mov axis.
                // let normal = Axis::other(mov.face().axis(), position.normal());

                // `i` and `j` are the axes that get rotated.
                // Specifically (p[i], p[j]) -> (p[j], -p[i])   (where `p` is position).
                let i = (mov.face().axis().u8() + 1) % 3;
                let j = (mov.face().axis().u8() + 2) % 3;

                // Here, with ternary coordinates, one is 0 and the other is either 1 or -1.
                // If `i` is 0, then `p[j]` stays as-is. Otherwise, `p[j]` gets flipped.
                let other_face = if i == position.normal().u8() {
                    // assert!(j == position.normal().u8());
                    let axis = Axis::from_u8(i);
                    let dir = position.direction_on_axis(Axis::from_u8(j)).flip();
                    Face::new(axis, dir)
                } else {
                    let axis = Axis::from_u8(j);
                    let dir = position.direction_on_axis(Axis::from_u8(i));
                    Face::new(axis, dir)
                };

                let faces = [other_face, mov.face()];
                EdgePosition::from_faces(faces)
            }
            (Amount::Reverse, Direction::Positive) | (Amount::Single, Direction::Negative) => {
                // `i` and `j` are the axes that get rotated.
                // Specifically (p[i], p[j]) -> (p[j], -p[i])   (where `p` is position).
                let i = (mov.face().axis().u8() + 2) % 3;
                let j = (mov.face().axis().u8() + 1) % 3;

                // Here, with ternary coordinates, one is 0 and the other is either 1 or -1.
                // If `i` is 0, then `p[j]` stays as-is. Otherwise, `p[j]` gets flipped.
                let other_face = if i == position.normal().u8() {
                    let axis = Axis::from_u8(i);
                    let dir = position.direction_on_axis(Axis::from_u8(j)).flip();
                    Face::new(axis, dir)
                } else {
                    let axis = Axis::from_u8(j);
                    let dir = position.direction_on_axis(Axis::from_u8(i));
                    Face::new(axis, dir)
                };

                let faces = [other_face, mov.face()];
                EdgePosition::from_faces(faces)
            }
        };

        let out = new_edge_pos.pick(&edges);

        // single Y-axis moves flip orientation.
        // This value is 1 if move is on x-axis, 0 otherwise.
        let is_on_z_axis = mov.face().axis().u8() >> 1;
        debug_assert!(if mov.face().axis() == Axis::Z {
            is_on_z_axis == 1
        } else {
            is_on_z_axis == 0
        });

        Edge {
            data: out.data ^ (is_on_z_axis << 4),
        }
    })
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b] = self.position().faces();
        write!(f, "{a}{b} ({})", if self.is_oriented() { '✓' } else { 'x' })
    }
}

impl fmt::Debug for EdgePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EdgePosition")
            .field("a", &self.a())
            .field("b", &self.b())
            .field("normal", &self.normal())
            .finish()
    }
}

impl fmt::Display for EdgePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Bodge
        let [a, b] = self.faces();
        write!(f, "{a}{b}")
    }
}

#[cfg(all(test, feature = "quickcheck"))]
mod tests {
    use quickcheck::{Arbitrary, Gen, TestResult, quickcheck};

    use super::*;

    impl Arbitrary for Edge {
        fn arbitrary(g: &mut Gen) -> Self {
            let normal = Axis::arbitrary(g);
            let a = Direction::arbitrary(g);
            let b = Direction::arbitrary(g);
            let orientation = bool::arbitrary(g);
            Edge {
                data: (a.u8() << 0)
                    + (b.u8() << 1)
                    + (normal.u8() << 2)
                    + ((orientation as u8) << 4),
            }
        }
    }
    impl Arbitrary for EdgePosition {
        fn arbitrary(g: &mut Gen) -> Self {
            let normal = Axis::arbitrary(g);
            let a = Direction::arbitrary(g);
            let b = Direction::arbitrary(g);
            EdgePosition {
                data: (a.u8() << 0) + (b.u8() << 1) + (normal.u8() << 2),
            }
        }
    }

    quickcheck! {
        fn from_faces_produces_edge_with_those_faces(f1: Face, f2: Face) -> TestResult {
            if f1.axis() == f2.axis() {
                TestResult::discard()
            } else {
                let faces =
                    EdgePosition::from_faces([f1, f2]).faces();

                TestResult::from_bool(
                    faces == [f1, f2] || faces == [f2, f1]
                )
            }
        }

        fn turn_distnace_distribution_is_1_6_1(position: EdgePosition) -> bool {
            let mut bins = [0, 0, 0];
            for other in EdgePosition::ALL {
                let diff = position.turn_distance(other);
                bins[diff as usize] += 1;
            }

            bins == [1, 6, 5]
        }
    }
}
