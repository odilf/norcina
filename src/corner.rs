use crate::{
    cube::Sticker,
    math::{Axis, Direction, Face},
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

    pub fn oriented_from_faces(faces: [Face; 3]) -> Self {
        let index = faces
            .into_iter()
            .map(|face| (face.direction().u8() << face.axis().u8()))
            .sum();

        Corner { data: index }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CornerPosition(Corner);

impl CornerPosition {
    pub fn from_faces(faces: [Face; 3]) -> Self {
        let index = faces
            .into_iter()
            .map(|face| (face.direction().u8() << face.axis().u8()))
            .sum();

        CornerPosition(Corner { data: index })
    }

    pub fn pick(self, corners: [Corner; 8]) -> Corner {
        corners[self.0.data as usize]
    }
}

pub fn sticker(corner: Corner, position: CornerPosition, face: Face) -> Sticker {
    let axis = Axis::from_u8((face.axis().u8() + corner.orientation().u8()) % 3);
    Face::new(axis, position.0.direction_on_axis(axis))
}
