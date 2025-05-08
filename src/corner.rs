use std::{array, fmt};

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
                && faces[2].axis() != faces[0].axis()
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
}

pub fn sticker(corner: Corner, position: CornerPosition, face: Face) -> Sticker {
    // TODO: Actually use orientation.
    let axis = face.axis();
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
        let i = match mov.amount() {
            Amount::Single => {
                let a = 2;
                let b = 1;
                let temp = ((i >> a) ^ (i >> b)) & 0b1;
                i ^ ((temp << a) | ((temp ^ 0b1) << b))
            }
            Amount::Double => i ^ (0b111 ^ mask),
            Amount::Reverse => {
                let a = 2;
                let b = 1;
                let temp = ((i >> a) ^ (i >> b)) & 0b1;
                i ^ (((temp ^ 0b1) << a) | (temp << b))
            }
        };

        corners[i as usize]
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
