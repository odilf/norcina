use barbaroja::{alg, cube::Cube, mov::algs};

fn main() {
    // let cube = Cube::SOLVED.mov(algs::T);
    let cube = Cube::SOLVED.mov(alg!(U DP));
    let s = cube.to_string();
    dbg!(cube);
    println!("{s}");
}
