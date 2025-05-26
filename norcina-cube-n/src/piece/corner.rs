use crate::{
    Sticker,
    math::{Axis, Direction, Face},
    mov::{Amount, Move},
};
use std::{array, fmt, mem::transmute};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Corner {
    /// Packed field `000oozyx`
    ///
    /// Invariants:
    /// - Orientation is always 0, 1 or 2.
    /// - Three most significant bits are always 0.
    data: u8,
}

impl Corner {
    pub const SOLVED: [Self; 8] = [
        Corner::solved(0),
        Corner::solved(1),
        Corner::solved(2),
        Corner::solved(3),
        Corner::solved(4),
        Corner::solved(5),
        Corner::solved(6),
        Corner::solved(7),
    ];

    #[inline]
    pub const fn x(self) -> Direction {
        Direction::from_bool(self.data & 0b001 != 0)
    }

    #[inline]
    pub const fn y(self) -> Direction {
        Direction::from_bool(self.data & 0b010 != 0)
    }

    #[inline]
    pub const fn z(self) -> Direction {
        Direction::from_bool(self.data & 0b100 != 0)
    }

    #[inline]
    pub const fn orientation(self) -> Axis {
        let v = (self.data >> 3) & 0b11;
        debug_assert!(v < 3);
        // SAFETY: orientation bits are guaranteed to be either 0, 1 or 2.
        unsafe { Axis::from_u8_unchecked(v) }
    }

    #[inline]
    pub const fn direction_on_axis(self, axis: Axis) -> Direction {
        Direction::from_bool(self.data >> axis.u8() & 0b1 != 0)
    }

    pub const fn solved(index: u8) -> Corner {
        assert!(index < 8);
        Corner { data: index }
    }

    /// Whether the piece is on the given face.
    #[inline]
    pub fn on_face(self, face: Face) -> bool {
        self.direction_on_axis(face.axis()) == face.direction()
    }

    #[inline]
    pub const fn position(self) -> CornerPosition {
        // SAFETY: `CornerPosition` and `Corner` have the same single u8 layout, except the orientation bits, which
        // we strip out.
        unsafe { transmute(self.data & 0b00111) }
    }

    /// Returns a possible set of 8 corners.
    ///
    /// The sum of the orientations is a multiple of 3.
    pub fn random(rng: &mut impl rand::Rng) -> [Corner; 8] {
        use rand::seq::SliceRandom;

        let mut out = Self::SOLVED;
        out.shuffle(rng);

        let mut total_orientaiton = 0;
        for corner in &mut out[0..7] {
            let orientation = rng.random_range(0..3);
            corner.data += orientation << 3;
            total_orientaiton += orientation;
        }

        // TODO: Does this work?
        out[7].data += (total_orientaiton.wrapping_neg() % 3) << 3;

        out
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CornerPosition {
    data: u8,
}

impl CornerPosition {
    #[inline]
    pub const fn x(self) -> Direction {
        Direction::from_bool(self.data & 0b001 != 0)
    }

    #[inline]
    pub const fn y(self) -> Direction {
        Direction::from_bool(self.data & 0b010 != 0)
    }

    #[inline]
    pub const fn z(self) -> Direction {
        Direction::from_bool(self.data & 0b100 != 0)
    }

    pub fn from_faces(faces: [Face; 3]) -> Self {
        debug_assert!(
            faces[0].axis() != faces[1].axis()
                && faces[1].axis() != faces[2].axis()
                && faces[2].axis() != faces[0].axis(),
            "Faces don't form a corner ({faces:?})"
        );

        let index = faces
            .into_iter()
            .map(|face| (face.direction().u8() << face.axis().u8()))
            .sum();

        CornerPosition { data: index }
    }

    pub const fn faces(self) -> [Face; 3] {
        [
            Face::new(Axis::X, self.x()),
            Face::new(Axis::Y, self.y()),
            Face::new(Axis::Z, self.z()),
        ]
    }

    // SAFETY: index must be between 0 and 7.
    pub const unsafe fn from_index_unchecked(index: u8) -> Self {
        debug_assert!(index < 8);
        // SAFETY: Numbers between 0 and 7 are valid corner positions.
        unsafe { transmute(index) }
    }

    pub const fn from_index(index: u8) -> Self {
        assert!(index < 8);
        CornerPosition { data: index }
    }

    pub const fn pick(self, corners: [Corner; 8]) -> Corner {
        corners[self.data as usize]
    }

    pub fn contains_face(self, face: Face) -> bool {
        (self.data >> face.axis().u8()) & 0b1 == face.direction().u8()
    }

    /// xor of all position bits, either 0 or 1
    const fn parity(self) -> u8 {
        (self.data ^ (self.data >> 1) ^ (self.data >> 2)) & 0b1
    }

    /// The minimum number of turns to get from `self` to `other`.
    ///
    /// There is
    /// - 1 position where this value is 0 (itself),
    /// - 6 positions where the value is 1 and
    /// - 1 position where the value is 2 (the opposite corner).
    pub fn turn_distance(self, other: CornerPosition) -> u8 {
        let diff_coords = ((self.data ^ other.data) & 0b111).count_ones() as u8;
        // number of different coords -> output
        // 0 0b00 -> 0
        // 1 0b01 -> 1
        // 2 0b10 -> 1
        // 3 0b11 -> 2
        // huh, just count ones... again.
        // Or, (x + 1) / 2, which I'm pretty sure I've done elsewhere...
        // diff_coords.count_ones() as u8
        (diff_coords + 1) / 2
    }

    pub const ALL: [CornerPosition; 8] = [
        CornerPosition::from_index(0),
        CornerPosition::from_index(1),
        CornerPosition::from_index(2),
        CornerPosition::from_index(3),
        CornerPosition::from_index(4),
        CornerPosition::from_index(5),
        CornerPosition::from_index(6),
        CornerPosition::from_index(7),
    ];

    pub const fn u8(self) -> u8 {
        // TODO: Transmute?
        self.data
    }

    pub fn with_orientation(self, orientation: Axis) -> Corner {
        Corner {
            data: self.data + (orientation.u8() << 3),
        }
    }
}

pub fn sticker(corner: Corner, position: CornerPosition, face: Face) -> Sticker {
    // First, we figure out the index of the face's axis for the given position:
    let face_index = if position.parity() == 0 {
        (3 + face.axis().u8() - corner.orientation().u8()) % 3
    } else {
        (6 - face.axis().u8() - corner.orientation().u8()) % 3
    };

    // Then, we take that index from the corner.
    let axis = Axis::from_u8(if corner.position().parity() == 0 {
        face_index
    } else {
        (3 - face_index) % 3
    });

    // let axis = Axis::from_u8((face.axis().u8() + axis_diff) % 3);
    Face::new(axis, corner.direction_on_axis(axis))
}

pub fn move_pieces(corners: [Corner; 8], mov: Move) -> [Corner; 8] {
    array::from_fn(|i| {
        let position = CornerPosition::from_index(i as u8);
        if !position.contains_face(mov.face()) {
            return position.pick(corners);
        }

        // TODO: Maybe this should be a method in position?
        // TODO: Surely there is a way to do this with less branching.
        let (a, b) = match (mov.amount(), mov.face().direction()) {
            (Amount::Single, Direction::Positive) | (Amount::Reverse, Direction::Negative) => {
                ((mov.axis().u8() + 1) % 3, (mov.axis().u8() + 2) % 3)
            }
            (Amount::Reverse, Direction::Positive) | (Amount::Single, Direction::Negative) => {
                ((mov.axis().u8() + 2) % 3, (mov.axis().u8() + 1) % 3)
            }
            (Amount::Double, _) => {
                let mask = 0b1 << mov.axis().u8();
                return corners[i ^ (0b111 ^ mask) as usize];
            }
        };

        // Do rotation (a, b) -> (b, -a);
        let temp = ((i >> a) ^ (i >> b)) & 0b1;
        let i = i ^ (((temp ^ 0b1) << a) | (temp << b));

        // Corner twists:
        // - Unchanged if move is on orientation axis
        // - Otherwise, conjecture for how much to add.
        // For the first part, this value is 0 if move is on x-axis, 1 otherwise.
        let is_not_on_x_axis = (mov.axis().u8() + 1) / 2;
        assert!(
            if mov.face().axis() == Axis::X {
                is_not_on_x_axis == 0
            } else {
                is_not_on_x_axis == 1
            },
            "{is_not_on_x_axis} but is on {:?}",
            mov.face().axis()
        );

        // For the second part, my conjecture is that it adds 2 if the
        // xor of the position is 0, otherwise 1
        let orientation_diff = is_not_on_x_axis
            << (position.parity() as u8 ^ (mov.amount().u8() & 0b1) ^ (mov.axis().u8() >> 1));

        // TODO: Maybe we can inline this mo'
        let mut out = corners[i as usize];
        // TODO: Does this work how I think?
        out.data = (out.data + (orientation_diff << 3)) % (3 << 3);
        out
    })
}

impl fmt::Display for Corner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c] = self.position().faces();
        let o = self.orientation();
        write!(f, "{a}{b}{c} ({})", o.u8())
    }
}

impl fmt::Display for CornerPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c] = self.faces();
        write!(f, "{a}{b}{c}")
    }
}

#[cfg(feature = "quickcheck")]
mod quickcheck_impl {
    use super::*;

    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Corner {
        fn arbitrary(g: &mut Gen) -> Self {
            let x = Direction::arbitrary(g);
            let y = Direction::arbitrary(g);
            let z = Direction::arbitrary(g);
            let orientation = Axis::arbitrary(g);
            Corner {
                data: (x.u8() << 0) + (y.u8() << 1) + (z.u8() << 2) + (orientation.u8() << 3),
            }
        }
    }

    impl Arbitrary for CornerPosition {
        fn arbitrary(g: &mut Gen) -> Self {
            let x = Direction::arbitrary(g);
            let y = Direction::arbitrary(g);
            let z = Direction::arbitrary(g);
            CornerPosition {
                data: (x.u8() << 0) + (y.u8() << 1) + (z.u8() << 2),
            }
        }
    }
}

#[cfg(all(test, feature = "quickcheck"))]
mod tests {
    use quickcheck::quickcheck;

    use super::*;

    quickcheck! {
        fn from_faces_produces_corner_with_those_faces(d1: Direction, d2: Direction, d3: Direction) -> bool {
            let faces = [
                Face::new(Axis::X, d1),
                Face::new(Axis::Y, d2),
                Face::new(Axis::Z, d3),
            ];

            let new_faces = CornerPosition::from_faces(faces).faces();

            faces == new_faces
        }

        fn turn_distnace_distribution_is_1_6_1(position: CornerPosition) -> bool {
            let mut bins = [0, 0, 0];
            for other in CornerPosition::ALL {
                let diff = position.turn_distance(other);
                bins[diff as usize] += 1;
            }

            bins == [1, 6, 1]
        }
    }
}
