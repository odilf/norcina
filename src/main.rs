use barbaroja::{alg, cube::Cube, mov::algs};

fn main() {
    // let cube = Cube::SOLVED.mov(algs::T);
    let cube = Cube::SOLVED.mov(alg!(R));
    let s = cube.to_string();
    println!("{s}");
}
