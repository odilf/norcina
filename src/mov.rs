use std::ops;

use crate::math::{Axis, Direction, Face};

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

pub mod algs {
    use super::Move;

    pub const SEXY: [Move; 4] = alg!(R U RP UP);
    pub const SLEDGEHAMMER: [Move; 4] = alg!(RP F R FP);
    pub const T: [Move; 14] = alg!(R U RP UP RP F R2 UP RP UP R U RP FP);
    pub const J: [Move; 13] = alg!(R U RP F R U RP UP RP FP R2 UP RP);

    pub const CHECKER: [Move; 6] = alg!(R2 L2 U2 D2 F2 B2);

    // TODO: Concat or extend algs
    // pub const J_AUF: [Move; 14] = [J, alg!(UP)].concat();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::Cube;
    use quickcheck::{Arbitrary, Gen, quickcheck};

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

    quickcheck! {
        fn fn_move_constructor_and_accessors_maintain_values(face: Face, amount: Amount) -> bool {
            let mov = Move::new(face, amount);
            mov.face() == face && mov.amount() == amount
        }

        fn fn_double_double_identity(cube: Cube, face: Face) -> bool {
            let mov = Move::new(face, Amount::Double);
            cube.mov([mov, mov]) == cube
        }

        fn fn_rrp_identity(cube: Cube) -> bool {
            cube.mov(alg!(R RP)) == cube
        }

        fn fn_move_reverse_identity(cube: Cube, face: Face) -> bool {
            let m1 = Move::new(face, Amount::Single);
            let m2 = Move::new(face, Amount::Reverse);
            cube.mov([m1, m2]) == cube
        }

        fn fn_double_t_identity(cube: Cube) -> bool {
            cube.mov(alg!(R RP)) == cube
        }

        fn fn_single_double_equals_reverse(cube: Cube) -> bool {
            cube.mov(alg!(R R2)) == cube.mov(alg!(RP))
        }
    }
}
