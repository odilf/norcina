//! Implementation of Kociemba's algorithm.
//!
//! # Resources
//! - Kociemba's webpage: https://web.archive.org/web/20150226041111/http://kociemba.org/cube.htm
//! - Prunte table in more detail: https://cube20.org/src/phase1prune.pdf
//! - Prune table reference implementation: https://qiita.com/7y2n/items/55abb991a45ade2afa28

use super::SearchSolution;
use crate::{Cube, Move, search::search_idastar};
use norcina_cube_n::{math::Axis, piece::edge::EdgePosition};

/// Moves that stay in G1.
pub const G1_MOVES: [Move; 10] = {
    use norcina_cube_n::mov::moves::*;
    [U, U2, UP, D, D2, DP, R2, L2, F2, B2]
};

pub fn solve(cube: Cube) -> SearchSolution {
    let prune_table = PruneTable::load_or_generate();
    solve_with_table(cube, &prune_table)
}

pub fn solve_with_table(cube: Cube, prune_table: &PruneTable) -> SearchSolution {
    let phase1_sol = solve_to_g1(cube, &prune_table);
    debug_assert!(is_in_g1(phase1_sol.final_state()));
    let phase2_sol = solve_from_g1(phase1_sol.final_state(), &prune_table);
    phase1_sol.concat(phase2_sol)
}

/// Takes a scrambled cube and finds the closest algorithm to a state in the
// "G1" subset. Any state in this subset can be solved using just U, D, R2, L2,
// F2 and B2 moves.
pub fn solve_to_g1(cube: Cube, prune_table: &PruneTable) -> SearchSolution {
    search_idastar(
        cube,
        |cube| prune_table.phase1_distance_heuristic(cube),
        is_in_g1,
    )
}

/// Takes a cube in a G1 state and solves it.
pub fn solve_from_g1(cube: Cube, prune_table: &PruneTable) -> SearchSolution {
    debug_assert!(is_in_g1(cube));
    search_idastar(
        cube,
        |cube| prune_table.phase2_distance_heuristic(cube),
        Cube::is_solved,
    )
}

/// A cube is in G1 if:
/// 1. All of the corners are oriented
/// 2. All of the edges are oriented
/// 3. All the y-normal pieces are in their slice.
pub fn is_in_g1(cube: Cube) -> bool {
    let corners_are_oriented = || cube.corners.iter().all(|&corner| corner.is_oriented());
    let edges_are_oriented = || cube.edges.iter().all(|&edge| edge.is_oriented());
    let y_edges_are_in_y_slice = || {
        cube.edges.iter().enumerate().all(|(i, &edge)| {
            let position = EdgePosition::from_index(i as u8);
            (position.normal() == Axis::Y) == (edge.position().normal() == Axis::Y)
        })
    };

    corners_are_oriented() && edges_are_oriented() && y_edges_are_in_y_slice()
}

pub use prune_table::PruneTable;
mod prune_table;

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    #[test]
    fn solved_cube_is_in_g1() {
        assert!(is_in_g1(Cube::SOLVED))
    }

    quickcheck! {
        fn moves_in_g1_stay_in_g1(moves: Vec<u8>) -> bool {
            let state = Cube::SOLVED.mov(moves.into_iter().map(|mov_idx| G1_MOVES[(mov_idx % 10) as usize]));
            is_in_g1(state)
        }
    }
}
