#[cfg(feature = "lut_heuristic")]
mod lut_heuristic;
#[cfg(feature = "lut_heuristic")]
pub use lut_heuristic::TableHeuristic;

#[cfg(feature = "kociemba")]
pub mod kociemba;
#[cfg(feature = "kociemba")]
pub use kociemba::solve as solve_kociemba;

use crate::{Alg, Cube, Move};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSolution {
    pub states: Vec<Cube>,
}

impl SearchSolution {
    /// Finds the connecting move between two states.
    fn connect(start: Cube, end: Cube) -> Option<Move> {
        Move::iter().find(|&mov| start.mov_single(mov) == end)
    }

    pub fn moves(self) -> Vec<Move> {
        self.states
            .windows(2)
            .map(|window| Self::connect(window[0], window[1]).unwrap())
            .collect()
    }

    pub fn alg(self) -> Alg {
        Alg {
            moves: self.moves(),
        }
    }

    pub fn final_state(&self) -> Cube {
        *self.states.last().unwrap()
    }

    fn concat(mut self, other: SearchSolution) -> SearchSolution {
        assert_eq!(self.final_state(), other.states[0]);

        // Skip the first state since the last from `self` is the first from `other`.
        self.states.extend_from_slice(&other.states[1..]);
        self
    }
}

pub fn search_bfs(initial_state: Cube, mut goal: impl FnMut(Cube) -> bool) -> SearchSolution {
    let states = pathfinding::directed::bfs::bfs(
        &initial_state,
        |cube| cube.neighbors().map(|(_, state)| state),
        |state| goal(*state),
    )
    .expect("Search space won't be exhausted");

    SearchSolution { states }
}

pub fn solve_bfs(state: Cube) -> SearchSolution {
    search_bfs(state, Cube::is_solved)
}

pub fn search_idastar(
    initial_state: Cube,
    mut heuristic: impl FnMut(Cube) -> u8,
    mut goal: impl FnMut(Cube) -> bool,
) -> SearchSolution {
    let (states, _cost) = pathfinding::directed::idastar::idastar(
        &initial_state,
        |cube| cube.neighbors().map(|(_mov, state)| (state, 1)),
        |cube| heuristic(*cube),
        |cube| goal(*cube),
    )
    .expect("Search space won't be exhausted.");

    SearchSolution { states }
}

pub fn solve_manhattan(state: Cube) -> SearchSolution {
    search_idastar(state, manhattan_distance, Cube::is_solved)
}

// TODO: How big does the return value need to be?
pub fn manhattan_distance(state: Cube) -> u8 {
    let c: u8 = state
        .corners()
        .map(|(position, state)| position.turn_distance(state.position()))
        .sum();
    let e: u8 = state
        .edges()
        .map(|(position, state)| position.turn_distance(state.position()))
        .sum();

    c + e
}
