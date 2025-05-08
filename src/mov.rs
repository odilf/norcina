use std::ops;

use crate::math::{Direction, Face};

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
        match x {
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Reverse,
            _ => panic!("Invalid amount."),
        }
    }
}

impl ops::Mul<Direction> for Amount {
    type Output = Amount;
    fn mul(self, rhs: Direction) -> Self::Output {
        // TODO: Is it possible to remove this branch?
        match rhs {
            Direction::Positive => self,
            Direction::Negative => Self::from_u8(4 - self.u8()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    /// Packed field: `----aaff`
    data: u8,
}

impl Move {
    pub const fn new(face: Face, amount: Amount) -> Move {
        Self {
            data: face.u8() + amount.u8() << 2,
        }
    }

    pub const fn face(self) -> Face {
        Face::from_u8(self.data & 0b11)
    }

    pub const fn amount(self) -> Amount {
        Amount::from_u8(self.data >> 2 & 0b11)
    }
}

#[macro_export]
macro_rules! alg {
    (@ $mov:tt) => { $mov };

    ($($mov:tt)*) => {{
        use $crate::mov::moves::*;
        [$(alg!(@ $mov)),*]
    }};

}

pub mod moves {
    macro_rules! generate_moves {
        // ($face:ident, $amount:ident, $amount_name:tt) => {
        //     pub const $face: Move = Move::new($face, $amount);
        // };

        // ($face:ident, [$($amounts:ident)*], [$($amount_names:tt)*]) => {
        //     $(
        //         generate_moves!($face, $amounts, $amount_names);
        //     )*
        // };
        (@amount 1) => { Amount::Single };
        (@amount 2) => { Amount::Double };
        (@amount 3) => { Amount::Reverse };

        (@face R) => { Face::R };
        (@face U) => { Face::U };
        (@face F) => { Face::F };
        (@face L) => { Face::L };
        (@face D) => { Face::D };
        (@face B) => { Face::B };

        ([$($face:tt $amount:tt $name:ident),*]) => {
            $(
                pub const $name: Move = Move::new(
                    generate_moves!(@face $face),
                    generate_moves!(@amount $amount)
                );
            )*
        };
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

#[cfg(test)]
mod tests {
    use crate::cube::Cube;
    use quickcheck::quickcheck;

    quickcheck! {
        fn fn_r2r2_identity(cube: Cube) -> bool {
            cube.mov(alg!(R2 R2)) == cube
        }
    }
}
