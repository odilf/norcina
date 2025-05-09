use barbaroja::{cube::Cube, mov::algs};

fn main() {
    let cube = Cube::SOLVED.mov(algs::T);
    let s = cube.to_string();
    dbg!(cube);
    println!("{s}");

    for mov in barbaroja::mov::Move::iter() {
        println!("{mov}");
        println!("{}", Cube::SOLVED.mov_single(mov))
    }
}
