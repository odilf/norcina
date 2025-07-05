//! Functionallity that is common to all `norcina` crates.

mod event;
pub use event::Event;

mod alg;
pub use alg::Alg;

pub mod mov;
pub use mov::Move;

pub mod math;

pub mod types;
