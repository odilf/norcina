use core::fmt;
use std::{
    fmt::Write,
    ops::{Deref, DerefMut},
};

use crate::mov::InvertibleMove;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Alg<M> {
    pub moves: Vec<M>,
}

impl<M> Alg<M> {
    pub fn reverse(&mut self)
    where
        M: InvertibleMove,
    {
        self.moves.reverse();
        for mov in &mut self.moves {
            *mov = mov.inverse();
        }
    }

    pub fn reversed(mut self) -> Alg<M>
    where
        M: InvertibleMove,
    {
        self.reverse();
        self
    }
}

impl<M: fmt::Display> fmt::Display for Alg<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, mov) in self.moves.iter().enumerate() {
            if i > 0 {
                f.write_char(' ')?;
            }
            write!(f, "{mov}")?;
        }

        Ok(())
    }
}

impl<M> Deref for Alg<M> {
    type Target = [M];
    fn deref(&self) -> &Self::Target {
        &self.moves
    }
}

impl<M> DerefMut for Alg<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.moves
    }
}
