use std::fmt;

use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Move {}

#[enum_dispatch]
pub trait InvertibleMove: Move {
    /// The move that undoes `self`.
    fn inverse(&self) -> Self;
}

#[enum_dispatch]
pub trait DisplayMove: Move + fmt::Display {}
impl<M: Move + fmt::Display> DisplayMove for M {}
