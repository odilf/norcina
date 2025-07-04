use norcina_cube3::{Cube, search::kociemba};
use rand::{SeedableRng, rngs::SmallRng};

fn main() {
    let mut rng = SmallRng::seed_from_u64(123);
    let cube = Cube::random_with_rng(&mut rng);
    println!("{cube}");
    let solution = kociemba::solve(cube).alg();
    println!("Solution is {}.", solution);
    println!("Therefore, scramble is {}.", solution.reversed());
}
