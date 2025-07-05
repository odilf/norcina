use super::Vertex;
use crate::mov::CoreMove;
use norcina_core::types::Orientation3;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Centers {
    // Packed struct: BBLLUURR
    data: u8,
}

impl Centers {
    pub const SOLVED: Self = Self { data: 0 };

    /// The orientation of the tip at the given vertex, relative to the
    /// corresponding center.
    ///
    /// 0 indicates solved, 1 indicates it's one clockwise rotation from
    /// solved, 2 indicates two clockwise rotations (or, equivalently, one
    /// counterclockwise rotation).
    ///
    /// That is, every clockwise move increases orientation by 1.
    #[inline(always)]
    pub const fn orientation_of(self, vertex: Vertex) -> Orientation3 {
        Orientation3::from_u8((self.data >> (vertex.u8() * 2)) & 0b11)
    }

    // TODO: Make const
    pub fn mov(self, mov: CoreMove) -> Self {
        let orientation_delta = mov.amount().u8() + 1;
        let new_orientation = ((self.data >> (mov.vertex().u8() * 2)) + orientation_delta) % 3;

        // TODO: Is there a more consice way to overwrite using bit twiddling?
        Self {
            data: self.data
                // Clear the old bits
                & (!(0b11 << (mov.vertex().u8() * 2)))
                // Fill in with new bits
                | (new_orientation << (mov.vertex().u8() * 2)),
        }
    }
}
