use std::{fmt, ops};

use crate::math::{Axis, Direction, Face};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Amount {
    Single = 1,
    Double = 2,
    Reverse = 3,
}

impl Amount {
    #[inline]
    pub const fn u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn from_u8(x: u8) -> Self {
        // TODO: Transmute and stuff
        match x {
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Reverse,
            _ => panic!("Invalid amount."),
        }
    }

    /// The amount that does the opposite of itself.
    pub const fn reverse(self) -> Amount {
        // TODO: This can be done more efficiently with bit-twiddling, but is it worth it...
        match self {
            Amount::Single => Amount::Reverse,
            Amount::Double => Amount::Double,
            Amount::Reverse => Amount::Single,
        }
    }

    pub fn iter() -> impl Iterator<Item = Amount> {
        [Amount::Single, Amount::Double, Amount::Reverse].into_iter()
    }
}

impl ops::Mul<Direction> for Amount {
    type Output = Amount;
    fn mul(self, rhs: Direction) -> Self::Output {
        // TODO: Is it possible to remove this branch?
        match rhs {
            Direction::Positive => self,
            #[allow(clippy::suspicious_arithmetic_impl)]
            Direction::Negative => Self::from_u8(4 - self.u8()),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move {
    /// Packed field: `---aafff`
    data: u8,
}

impl Move {
    pub const fn new(face: Face, amount: Amount) -> Move {
        Self {
            data: face.u8() + (amount.u8() << 3),
        }
    }

    pub const fn face(self) -> Face {
        Face::from_u8(self.data & 0b111)
    }

    pub const fn amount(self) -> Amount {
        Amount::from_u8(self.data >> 3)
    }

    pub const fn axis(self) -> Axis {
        self.face().axis()
    }

    /// Enumerates all possible moves.
    pub fn iter() -> impl Iterator<Item = Self> {
        Face::iter().flat_map(|face| Amount::iter().map(move |amount| Move::new(face, amount)))
    }

    pub const ALL: [Move; 18] = {
        use moves::*;
        [
            R, R2, RP, U, U2, UP, F, F2, FP, L, L2, LP, D, D2, DP, B, B2, BP,
        ]
    };
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field("face", &self.face())
            .field("amount", &self.amount())
            .finish()
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let amount_str = match self.amount() {
            Amount::Single => " ",
            Amount::Double => "2",
            Amount::Reverse => "'",
        };

        write!(f, "{}{}", self.face(), amount_str)
    }
}

impl norcina_core::Move for Move {}
impl norcina_core::mov::InvertibleMove for Move {
    fn inverse(&self) -> Self {
        // TODO: This can be done more efficiently with bit-twiddling.
        Self::new(self.face(), self.amount().reverse())
    }
}

pub mod moves {
    macro_rules! generate_moves {
        ([$($face:tt $amount:tt $name:ident),*]) => {
            $(
                pub const $name: Move = Move::new(
                    generate_moves!(@face $face),
                    generate_moves!(@amount $amount)
                );
            )*
        };

        (@amount 1) => { Amount::Single };
        (@amount 2) => { Amount::Double };
        (@amount 3) => { Amount::Reverse };

        (@face R) => { Face::R };
        (@face U) => { Face::U };
        (@face F) => { Face::F };
        (@face L) => { Face::L };
        (@face D) => { Face::D };
        (@face B) => { Face::B };
    }

    use super::Amount;
    use super::Face;
    use super::Move;

    generate_moves!([
        R 1 R, R 2 R2, R 3 RP,
        U 1 U, U 2 U2, U 3 UP,
        F 1 F, F 2 F2, F 3 FP,
        L 1 L, L 2 L2, L 3 LP,
        D 1 D, D 2 D2, D 3 DP,
        B 1 B, B 2 B2, B 3 BP
    ]);
}

#[macro_export]
macro_rules! alg {
    (@ $mov:tt) => { $mov };

    ($($mov:tt)*) => {{
        use $crate::mov::moves::*;
        [$(alg!(@ $mov)),*]
    }};
}

#[cfg(feature = "quickcheck")]
mod quickcheck_impl {
    use super::*;

    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Amount {
        fn arbitrary(g: &mut Gen) -> Self {
            *g.choose(&[Amount::Single, Amount::Double, Amount::Reverse])
                .unwrap()
        }
    }

    impl Arbitrary for Move {
        fn arbitrary(g: &mut Gen) -> Self {
            let face = Face::arbitrary(g);
            let amount = Amount::arbitrary(g);
            Move::new(face, amount)
        }
    }
}
