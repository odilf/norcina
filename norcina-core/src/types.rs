use std::{mem, ops};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Positive = 0,
    Negative = 1,
}

impl Direction {
    #[inline(always)]
    pub const fn from_bool(value: bool) -> Self {
        // SAFETY: The valid bit-patterns for `bool` are 0b0 and 0b1, which are
        // precisely the valid values for Direction.
        unsafe { mem::transmute(value) }
    }

    #[inline(always)]
    pub const fn bool(self) -> bool {
        // SAFETY: The valid bit-patterns for `bool` are 0b0 and 0b1, which are
        // precisely the valid values for Direction.
        unsafe { mem::transmute(self) }
    }

    #[inline(always)]
    pub const fn u8(self) -> u8 {
        self as u8
    }

    /// Maps 0 -> Positive, everything else -> Negative.
    ///
    /// Generally if you have a bit that represents direction, you can mask that
    /// bit and use this method.
    #[inline(always)]
    pub const fn from_u8_any(value: u8) -> Direction {
        Direction::from_bool(value != 0)
    }

    /// Constructs a [`Direction`] from the final bit of the value.
    ///
    /// Equivalent to `Self::from_u8(value & 0b1)`.
    pub const fn from_u8_mod2(value: u8) -> Direction {
        // SAFETY: We only take the final bit, so the possible bitpatterns are
        // just 0 and 1.
        unsafe { Self::from_u8_unchecked(value & 0b1) }
    }

    /// # Safety
    ///
    /// `value` needs to be either 0 or 1.
    #[inline]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Direction {
        debug_assert!(value < 2);
        unsafe { mem::transmute(value) }
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

    #[inline]
    pub const fn from_i8_mod3(value: i8) -> Axis {
        // SAFETY: We have taken modulo 3, so the possible values are 0, 1 or 2
        // (`rem_eculid` always has positive results, so the `as u8` is also sound).
        unsafe { Axis::from_u8_unchecked(value.rem_euclid(3) as u8) }
    }

    /// # Safety
    ///
    /// Value needs to be either 0, 1 or 2.
    pub const unsafe fn from_u8_unchecked(axis_index: u8) -> Axis {
        debug_assert!(axis_index < 3);
        unsafe { mem::transmute(axis_index) }
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
    /// It is kind of "cross product"-y (as in, the reulst is perpendicualr to
    /// both of the input axes).
    pub const fn other(a: Axis, b: Axis) -> Axis {
        // TODO: Is this good?
        //    a - (b - a)
        // => 2a - b
        Axis::from_i8_mod3((2 * a.u8() as i8) - b.u8() as i8)
    }
}

/// A type representing an orientation that can only be in 3 distinct states.
///
/// # Invariants
///
/// This type is a single byte with a bitpattern of either 0, 1 or 2.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Orientation3 {
    data: u8,
}

impl Orientation3 {
    pub const ZERO: Self = Orientation3 { data: 0 };
    pub const ONE: Self = Orientation3 { data: 1 };
    pub const TWO: Self = Orientation3 { data: 2 };

    /// # Safety
    ///
    /// `orientation` must be either 0, 1 or 2.
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(orientation: u8) -> Self {
        debug_assert!(orientation < 3);
        // SAFETY: Precondition is that orientation is either 0, 1 or 2.
        unsafe { mem::transmute(orientation) }
    }

    #[inline]
    pub const fn from_u8_mod3(orientation: u8) -> Self {
        // TODO: Use `Self::from_u8_unchecked`
        Orientation3 {
            data: orientation % 3,
        }
    }

    #[inline]
    pub const fn from_i8_mod3(orientation: i8) -> Self {
        // SAFETY: `rem_euclid` produces a value above 0, so the `as u8` is
        // sound, and it produces a value less than `3`. Therefore, either 0, 1
        // or 2.
        unsafe { Self::from_u8_unchecked(orientation.rem_euclid(3) as u8) }
    }

    /// # Panics
    ///
    /// If `orientation` is not either 0, 1 or 2.
    pub const fn from_u8(orientation: u8) -> Self {
        assert!(orientation < 3);
        // TODO: Use `Self::from_u8_unchecked`
        Orientation3 { data: orientation }
    }

    #[inline(always)]
    pub const fn u8(self) -> u8 {
        self.data
    }

    pub const fn is_oriented(self) -> bool {
        self.data == 0
    }
}

impl ops::Neg for Orientation3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::from_u8(((self.data | (self.data << 2)) >> 1) & 0b11)
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
}
