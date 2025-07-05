use std::{fmt, mem};

use crate::piece::Vertex;

// TODO: Maybe make a struct with a u8...
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Amount {
    Single = 0,
    Reverse = 1,
}

impl Amount {
    #[inline(always)]
    pub const fn u8(self) -> u8 {
        self as u8
    }

    /// # Safety
    ///
    /// `data` must be either 0 or 1.
    pub const unsafe fn from_u8_unchecked(data: u8) -> Self {
        debug_assert!(data < 2);
        // SAFETY: Precodnition is that it is either 0 or 1.
        unsafe { mem::transmute(data) }
    }

    pub const fn from_u8(data: u8) -> Self {
        assert!(data < 2);
        // SAFETY: We just checked that `data` is less than 2, so it is either 0 or 1.
        unsafe { mem::transmute(data) }
    }

    pub const fn from_u8_mod2(data: u8) -> Amount {
        // SAFETY: We take modulo 2.
        unsafe { Self::from_u8_unchecked(data & 0b1) }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreMove {
    // Packed field: -----avv
    data: u8,
}

impl CoreMove {
    pub const fn new(vertex: Vertex, amount: Amount) -> Self {
        Self {
            data: vertex.u8() + (amount.u8() << 2),
        }
    }

    pub const ALL: [CoreMove; 8] = [
        CoreMove::new(Vertex::R, Amount::Single),
        CoreMove::new(Vertex::U, Amount::Single),
        CoreMove::new(Vertex::L, Amount::Single),
        CoreMove::new(Vertex::B, Amount::Single),
        CoreMove::new(Vertex::R, Amount::Reverse),
        CoreMove::new(Vertex::U, Amount::Reverse),
        CoreMove::new(Vertex::L, Amount::Reverse),
        CoreMove::new(Vertex::B, Amount::Reverse),
    ];

    #[inline(always)]
    pub const fn vertex(self) -> Vertex {
        Vertex::from_u8_mod4(self.data & 0b011)
    }

    #[inline(always)]
    pub const fn amount(self) -> Amount {
        Amount::from_u8_mod2(self.data >> 2)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    // Packed field: ----tavv
    data: u8,
}

impl Move {
    pub const R: Self = Self::new(Vertex::R, Amount::Single, false);
    pub const RP: Self = Self::new(Vertex::R, Amount::Reverse, false);
    pub const L: Self = Self::new(Vertex::L, Amount::Single, false);
    pub const LP: Self = Self::new(Vertex::L, Amount::Reverse, false);
    pub const B: Self = Self::new(Vertex::B, Amount::Single, false);
    pub const BP: Self = Self::new(Vertex::B, Amount::Reverse, false);
    pub const U: Self = Self::new(Vertex::U, Amount::Single, false);
    pub const UP: Self = Self::new(Vertex::U, Amount::Reverse, false);

    pub const fn new(vertex: Vertex, amount: Amount, tip: bool) -> Self {
        Self {
            data: vertex.u8() + (amount.u8() << 2) + ((tip as u8) << 3),
        }
    }
    pub const fn is_tip_move(self) -> bool {
        (self.data & 0b1000) != 0
    }

    pub const fn core(self) -> CoreMove {
        CoreMove {
            data: self.data & 0b0111,
        }
    }

    pub const fn toggle_tip(self) -> Self {
        Self {
            data: self.data ^ 0b1000,
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addendum = if self.core().amount() == Amount::Single {
            ' '
        } else {
            '\''
        };

        let main = match (self.core().vertex(), self.is_tip_move()) {
            (Vertex::U, false) => "U",
            (Vertex::U, true) => "u",
            (Vertex::L, false) => "L",
            (Vertex::L, true) => "l",
            (Vertex::B, false) => "B",
            (Vertex::B, true) => "b",
            (Vertex::R, false) => "R",
            (Vertex::R, true) => "r",
            (_, _) => unreachable!(),
        };

        write!(f, "{main}{addendum}")
    }
}
