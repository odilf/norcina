use std::{array, fmt};

use owo_colors::{OwoColorize, colors::css::Orange};

use crate::{
    cube::Sticker,
    math::{Axis, Direction, Face},
    mov::{Amount, Move},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Corner {
    /// Packed field `---oozyx`
    data: u8,
}

impl Corner {
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
        Axis::from_u8((self.data >> 3) % 4)
    }

    #[inline]
    pub const fn direction_on_axis(self, axis: Axis) -> Direction {
        Direction::from_bool(self.data >> axis.u8() & 0b1 != 0)
    }

    pub const fn solved(index: u8) -> Corner {
        assert!(index < 8);
        Corner { data: index }
    }

    pub fn faces(self) -> [Face; 3] {
        [
            Face::new(Axis::X, self.x()),
            Face::new(Axis::Y, self.y()),
            Face::new(Axis::Z, self.z()),
        ]
    }

    /// Whether the piece is on the given face.
    #[inline]
    pub fn on_face(self, face: Face) -> bool {
        self.direction_on_axis(face.axis()) == face.direction()
    }

    pub const fn position(self) -> CornerPosition {
        // TODO: We can probably do a transmute here... But we should
        // figure out the specific invariants of whether the corner
        // position can have the orientation bits be non-zero or not.
        CornerPosition { data: self.data }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CornerPosition {
    data: u8,
}

impl CornerPosition {
    pub fn from_faces(faces: [Face; 3]) -> Self {
        assert!(
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

    pub fn from_index(index: u8) -> Self {
        assert!(index < 8);
        CornerPosition { data: index }
    }

    pub fn pick(self, corners: [Corner; 8]) -> Corner {
        corners[self.data as usize]
    }

    fn contains_face(self, face: Face) -> bool {
        (self.data >> face.axis().u8()) & 0b1 == face.direction().u8()
    }

    /// xor of all position bits, either 0 or 1
    const fn parity(self) -> u8 {
        (self.data ^ (self.data >> 1) ^ (self.data >> 2)) & 0b1
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
    let mask = 0b1 << mov.face().axis().u8();

    array::from_fn(|i| {
        let position = CornerPosition::from_index(i as u8);
        if !position.contains_face(mov.face()) {
            return position.pick(corners);
        }

        // TODO: Maybe this should be a method in position?
        // TODO: Surely there is a way to do this with less branching.
        let (a, b) = match (mov.amount(), mov.face().direction()) {
            (Amount::Single, Direction::Positive) | (Amount::Reverse, Direction::Negative) => (
                (mov.face().axis().u8() + 1) % 3,
                (mov.face().axis().u8() + 2) % 3,
            ),
            (Amount::Reverse, Direction::Positive) | (Amount::Single, Direction::Negative) => (
                (mov.face().axis().u8() + 2) % 3,
                (mov.face().axis().u8() + 1) % 3,
            ),
            (Amount::Double, _) => return corners[i ^ (0b111 ^ mask) as usize],
        };

        // Do rotation (a, b) -> (b, -a);
        let temp = ((i >> a) ^ (i >> b)) & 0b1;
        let i = i ^ (((temp ^ 0b1) << a) | (temp << b));

        // Corner twists:
        // - Unchanged if move is on orientation axis
        // - Otherwise, conjecture for how much to add.
        // For the first part, this value is 0 if move is on x-axis, 1 otherwise.
        let is_not_on_x_axis = (mov.face().axis().u8() + 1) / 2;
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
        let orientation_diff =
            is_not_on_x_axis << (position.parity() as u8 ^ (mov.amount().u8() & 0b1));

        // TODO: Maybe we can inline this mo'
        let mut out = corners[i as usize];
        // TODO: Does this work how I think?
        out.data = (out.data + (orientation_diff << 3)) % (3 << 3);
        out
    })
}

impl fmt::Display for Corner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c] = self.faces();
        let o = self.orientation();
        write!(f, "{a:?}{b:?}{c:?} ({o:?})")
    }
}

impl fmt::Display for CornerPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c] = Corner { data: self.data }.faces();
        write!(f, "{a:?}{b:?}{c:?}")
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{Arbitrary, Gen};

    use super::*;

    impl Arbitrary for Corner {
        fn arbitrary(g: &mut Gen) -> Self {
            let x = Direction::arbitrary(g);
            let y = Direction::arbitrary(g);
            let z = Direction::arbitrary(g);
            Corner {
                data: (x.u8() << 0) + (y.u8() << 1) + (z.u8() << 2),
            }
        }
    }
}
