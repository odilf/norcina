pub mod mov;

pub mod cube;

pub use cube::Cube;
pub use mov::algs;

pub use norcina_cube_n::mov::Move;

pub mod search;

pub type Alg = norcina_core::Alg<Move>;
