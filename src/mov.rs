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

    pub const R: Self = Move::new(Face::R, Amount::Single);
    pub const R2: Self = Move::new(Face::R, Amount::Double);
    pub const U: Self = Move::new(Face::U, Amount::Single);
    pub const U2: Self = Move::new(Face::U, Amount::Double);
    pub const F: Self = Move::new(Face::F, Amount::Single);
    pub const F2: Self = Move::new(Face::F, Amount::Double);
    pub const L: Self = Move::new(Face::L, Amount::Single);
    pub const L2: Self = Move::new(Face::L, Amount::Double);
    pub const D: Self = Move::new(Face::D, Amount::Single);
    pub const D2: Self = Move::new(Face::D, Amount::Double);
    pub const B: Self = Move::new(Face::B, Amount::Single);
    pub const B2: Self = Move::new(Face::B, Amount::Double);

    pub const fn face(self) -> Face {
        Face::from_u8(self.data & 0b11)
    }

    pub const fn amount(self) -> Amount {
        Amount::from_u8(self.data >> 2 & 0b11)
    }
}

#[cfg(test)]
mod tests {
    use crate::cube::Cube;

    use super::*;
    use quickcheck::quickcheck;

    quickcheck! {
        fn fn_r4_identity(cube: Cube) -> bool {
            cube.mov(Move::R2).mov(Move::R2) == cube
        }
    }
}
