use norcina_core::types::{Axis, Direction};
use std::fmt::{self, Write as _};

// TODO: Implement everything with transmuting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    /// Right
    R = 0,
    /// Up
    U = 1,
    /// Front
    F = 2,
    /// Left
    L = 4,
    /// Down
    D = 5,
    /// Back
    B = 6,
}

impl Face {
    pub const fn u8(self) -> u8 {
        self as u8
    }

    pub const fn new(axis: Axis, direction: Direction) -> Face {
        Face::from_u8(axis.u8() + (direction.u8() << 2))
    }

    pub const fn from_u8(index: u8) -> Face {
        assert!(index != 3 && index < 7);
        match index {
            0 => Face::R,
            1 => Face::U,
            2 => Face::F,
            4 => Face::L,
            5 => Face::D,
            6 => Face::B,
            _ => panic!("Invalid face index"),
        }
    }

    #[inline]
    pub const fn axis(self) -> Axis {
        Axis::from_u8(self.u8() & 0b011)
    }

    #[inline]
    pub const fn direction(self) -> Direction {
        Direction::from_bool(self.u8() & 0b100 != 0)
    }

    #[inline]
    pub const fn opposite(self) -> Face {
        Face::from_u8(self.u8() ^ 0b100)
    }

    /// The "cross product" of faces. I.e., takes two perpendicular faces and returns another perpendicular face
    /// that follows the right-hand rule.
    ///
    /// See also [`Axis::other`].
    ///
    /// # Panics
    ///
    /// If `self` and `rhs` are not perpendicular.
    pub fn cross(self, rhs: Face) -> Face {
        assert_ne!(self.axis(), rhs.axis());
        let axis = Axis::other(self.axis(), rhs.axis());
        let direction = Direction::from_bool(
            !(self.axis().next() == rhs.axis()) ^ self.direction().bool() ^ rhs.direction().bool(),
        );
        Face::new(axis, direction)
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [Face::R, Face::U, Face::F, Face::L, Face::D, Face::B].into_iter()
    }

    fn char(self) -> char {
        match self {
            Self::R => 'R',
            Self::U => 'U',
            Self::F => 'F',
            Self::L => 'L',
            Self::D => 'D',
            Self::B => 'B',
        }
    }
}

impl fmt::Display for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.char())
    }
}

#[cfg(feature = "quickcheck")]
mod quickcheck_impl {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Face {
        fn arbitrary(g: &mut Gen) -> Self {
            *g.choose([Face::R, Face::U, Face::F, Face::L, Face::D, Face::B].as_slice())
                .unwrap()
        }
    }
}

#[cfg(all(test, feature = "quickcheck"))]
mod tests {
    use super::*;
    use quickcheck::{TestResult, quickcheck};
    quickcheck! {
        fn other_axis_is_always_different_axis(a: Axis, b: Axis) -> TestResult {
            let other = Axis::other(a, b);
            if a == b {
                return TestResult::discard();
            }

            TestResult::from_bool(other != a && other != b)
        }

        // fn cross_product_follows_right_hand_rule(a: Face, direction: bool) -> bool {
        //     let b = if direction { a.next() } else { a.next().next() };
        //     a.cross(b).direction() == Direction::from_bool(direction)
        //     // let expected = if a.axis().next() == b.axis() { Direction::Positive } else { Direction::Negative };
        //     // a.cross(b).direction() == expected
        // }

        fn fn_axis_u8_is_always_between_0_and_2(x: Axis) -> bool {
            x.u8() < 3
        }

        fn fn_face_u8_has_always_direction_in_bit_3_and_axis_in_1_and_2(axis: Axis, direction: Direction) -> bool {
            let face = Face::new(axis, direction);
            (((face.u8() & 0b100) >> 2) == direction.u8())
                && (face.u8() & 0b11 == axis.u8())
        }
    }
}
