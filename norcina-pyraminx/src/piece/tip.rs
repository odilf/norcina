use super::{Centers, Vertex};
use crate::mov::CoreMove;
use norcina_core::types::Orientation3;

/// Tips hold literally the same information as tips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tips(Centers);

impl Tips {
    pub const SOLVED: Self = Self(Centers::SOLVED);

    /// See [`Centers::orientation_of`]
    #[inline(always)]
    pub const fn orientation_of(self, vertex: Vertex) -> Orientation3 {
        self.0.orientation_of(vertex)
    }

    /// See [`Centers::mov`]
    #[inline(always)]
    pub fn mov(self, mov: CoreMove) -> Self {
        Self(self.0.mov(mov))
    }

    #[inline(always)]
    pub fn as_centers(self) -> Centers {
        self.0
    }
}
