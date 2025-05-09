// TODO: Do this with bit manipulations and transmute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Positive = 0,
    Negative = 1,
}

impl Direction {
    #[inline]
    pub const fn from_bool(value: bool) -> Self {
        // TODO: Do this with bit manipulations and transmute
        if value {
            Direction::Negative
        } else {
            Direction::Positive
        }
    }

    #[inline]
    const fn bool(self) -> bool {
        // TODO: Do this with bit manipulations and transmute
        match self {
            Direction::Positive => false,
            Direction::Negative => true,
        }
    }

    #[inline]
    pub const fn u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn is_positive(self) -> bool {
        // TODO: Do this with bit manipulations and transmute
        matches!(self, Direction::Positive)
    }

    #[inline]
    pub const fn is_negative(self) -> bool {
        // TODO: Do this with bit manipulations and transmute
        matches!(self, Direction::Negative)
    }

    #[inline]
    pub const fn flip(self) -> Self {
        // TODO: Do this with bit manipulations and transmute
        match self {
            Direction::Positive => Direction::Negative,
            Direction::Negative => Direction::Positive,
        }
    }

    pub const fn from_u8(axis_offset: u8) -> Direction {
        // TODO: Do this with bit manipulations and transmute
        Self::from_bool(axis_offset != 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    #[inline]
    pub const fn next(self) -> Self {
        Axis::from_u8((self as u8 + 1) % 3)
    }

    #[inline]
    pub const fn prev(self) -> Axis {
        Axis::from_u8((self as u8 + 2) % 3)
    }

    #[inline]
    pub const fn from_u8(axis_index: u8) -> Axis {
        match axis_index {
            0 => Axis::X,
            1 => Axis::Y,
            2 => Axis::Z,
            _ => panic!("Axis index was greater than 3"),
        }
    }

    /// An axis that is neither `a` or `b`.
    ///
    /// It is kind of "cross product"-y. See also [`Face::cross`]
    pub const fn other(a: Axis, b: Axis) -> Axis {
        // TODO: Is this good?
        //    a - (b - a)
        // => 2a - b
        // => 2a + 3 - b (to prevent underflow)
        Axis::from_u8(((2 * a.u8()) + 3 - b.u8()) % 3)
    }

    pub const fn u8(self) -> u8 {
        self as u8
    }
}

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
        // match (axis, direction) {
        //     (Axis::X, Direction::Positive) => Face::R,
        //     (Axis::Y, Direction::Positive) => Face::U,
        //     (Axis::Z, Direction::Positive) => Face::F,
        //     (Axis::X, Direction::Negative) => Face::L,
        //     (Axis::Y, Direction::Negative) => Face::D,
        //     (Axis::Z, Direction::Negative) => Face::B,
        // }
    }

    pub const fn from_u8(index: u8) -> Face {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, TestResult, quickcheck};

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
