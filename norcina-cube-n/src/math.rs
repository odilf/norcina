use std::{
    fmt::{self, Write as _},
    mem::transmute,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Positive = 0,
    Negative = 1,
}

impl Direction {
    #[inline]
    pub const fn from_bool(value: bool) -> Self {
        // SAFETY: The valid bit-patterns for `bool` are 0b0 and 0b1, which are precisely the valid values for Direction.
        unsafe { transmute(value) }
    }

    #[inline]
    const fn bool(self) -> bool {
        // SAFETY: The valid bit-patterns for `bool` are 0b0 and 0b1, which are precisely the valid values for Direction.
        unsafe { transmute(self) }
    }

    #[inline]
    pub const fn u8(self) -> u8 {
        self as u8
    }

    /// Maps 0 -> Positive, everything else -> Negative.
    #[inline]
    pub const fn from_u8_any(value: u8) -> Direction {
        Direction::from_bool(value != 0)
    }

    /// # Safety
    ///
    /// `value` needs to be either 0 or 1.
    #[inline]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Direction {
        debug_assert!(value < 2);
        unsafe { transmute(value) }
    }

    /// Maps 0 -> Positive, 1 -> Negative.
    ///
    /// Panics value is other than 0 or 1.
    #[inline]
    pub const fn from_u8(value: u8) -> Direction {
        assert!(value < 2);
        unsafe { Self::from_u8_unchecked(value) }
    }

    #[inline]
    pub const fn flip(self) -> Self {
        unsafe { Self::from_u8_unchecked(self.u8() ^ 0b1) }
    }
}

impl std::ops::BitXor for Direction {
    type Output = Direction;
    fn bitxor(self, rhs: Self) -> Self::Output {
        // TODO: Check is unnecessary, pretty sure we know answer is valid statically.
        Self::from_u8(self.u8() ^ rhs.u8())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    #[inline]
    pub const fn u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn from_u8(value: u8) -> Axis {
        assert!(value < 3);
        // SAFETY: `Axis` is a u8 and the value can only be 0, 1 or 2.
        unsafe { Axis::from_u8_unchecked(value) }
    }

    #[inline]
    pub const fn from_u8_mod3(value: u8) -> Axis {
        // SAFETY: We have taken modulo 3, so the possible values are 0, 1 or 2.
        unsafe { Axis::from_u8_unchecked(value % 3) }
    }

    /// # Safety
    ///
    /// Value needs to be either 0, 1 or 2.
    pub const unsafe fn from_u8_unchecked(axis_index: u8) -> Axis {
        debug_assert!(axis_index < 3);
        unsafe { transmute(axis_index) }
    }

    #[inline]
    pub const fn next(self) -> Self {
        // TODO: Is there a more efficient way?
        // 00 -> 01
        // 01 -> 10
        // 10 -> 00
        Axis::from_u8_mod3(self.u8() + 1)
    }

    #[inline]
    pub const fn prev(self) -> Axis {
        // TODO: Is there a more efficient way?
        // 00 -> 10
        // 10 -> 01
        // 01 -> 00
        Axis::from_u8_mod3(self.u8() + 2)
    }

    /// An axis that is neither `a` or `b`.
    ///
    /// It is kind of "cross product"-y. See also [`Face::cross`]
    pub const fn other(a: Axis, b: Axis) -> Axis {
        // TODO: Is this good?
        //    a - (b - a)
        // => 2a - b
        // => 2a + 3 - b (to prevent underflow)
        Axis::from_u8_mod3((2 * a.u8()) + 3 - b.u8())
    }
}

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

    /// "cross product" of faces. I.e., takes two perpendicular faces and returns another perpendicular face
    /// that follows the right-hand rule.
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

    impl Arbitrary for Direction {
        fn arbitrary(g: &mut Gen) -> Self {
            *g.choose([Direction::Positive, Direction::Negative].as_slice())
                .unwrap()
        }
    }
    impl Arbitrary for Axis {
        fn arbitrary(g: &mut Gen) -> Self {
            *g.choose([Axis::X, Axis::Y, Axis::Z].as_slice()).unwrap()
        }
    }

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
