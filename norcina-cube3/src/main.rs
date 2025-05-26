use norcina_cube_n::alg;
use norcina_cube3::{Cube, search::solve_manhattan};

fn main() {
    let scramble = alg!(R U R2 U2 F2 BP);
    let cube = Cube::SOLVED.mov(scramble);

    let solution = solve_manhattan(cube);
    dbg!(solution);
}
