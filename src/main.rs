use barbaroja::{cube::Cube, mov::Move};

fn main() {
    // let cube = Cube::SOLVED.mov(Move::R).mov(Move::R);
    let cube = Cube::SOLVED;
    dbg!(cube);
    let s = cube.to_string();
    println!("{s}");
}
