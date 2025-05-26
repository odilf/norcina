use std::io;

fn main() -> io::Result<()> {
    norcina_cube3::search::cached_heuristic::generate_cache()
}
