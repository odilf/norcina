use norcina_pyraminx::{Pyraminx, mov::Move};

pub fn main() {
    let state = Pyraminx::SOLVED.mov([Move::LP]);
    // let state = Pyraminx::SOLVED.mov([Move::R]);
    // assert_ne!(state, Pyraminx::SOLVED);
    println!("{state}");
}
