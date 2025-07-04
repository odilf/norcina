use norcina_cube_n::alg;
use norcina_cube3::{Cube, search::solve_kociemba};

fn main() {
    let scramble = alg!(R U D F2 R L D2);
    let solution = solve_kociemba(Cube::SOLVED.mov(scramble));
    println!("Solution is {}", solution.alg());
}
