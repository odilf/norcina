#[cfg(feature = "lut_heuristic")]
mod lut_heuristic;
#[cfg(feature = "lut_heuristic")]
pub use lut_heuristic::TableHeuristic;

use crate::{Cube, Move};

fn connect(start: Cube, end: Cube) -> Option<Move> {
    Move::iter().find(|&mov| start.mov_single(mov) == end)
}

fn reconstruct_solution(state_path: Vec<Cube>) -> Vec<Move> {
    state_path
        .windows(2)
        .map(|window| connect(window[0], window[1]).unwrap())
        .collect()
}

pub fn search_bfs(initial_state: Cube, mut goal: impl FnMut(Cube) -> bool) -> Vec<Move> {
    let state_path = pathfinding::directed::bfs::bfs(
        &initial_state,
        |cube| cube.neighbors().map(|(_, state)| state),
        |state| goal(*state),
    )
    .expect("Search space won't be exhausted");

    reconstruct_solution(state_path)
}

pub fn solve_bfs(state: Cube) -> Vec<Move> {
    search_bfs(state, Cube::is_solved)
}

pub fn search_idastar(
    initial_state: Cube,
    mut heuristic: impl FnMut(Cube) -> u8,
    mut goal: impl FnMut(Cube) -> bool,
) -> Vec<Move> {
    let (state_path, _cost) = pathfinding::directed::idastar::idastar(
        &initial_state,
        |cube| cube.neighbors().map(|(_mov, state)| (state, 1)),
        |cube| heuristic(*cube),
        |cube| goal(*cube),
    )
    .expect("Search space won't be exhausted.");

    reconstruct_solution(state_path)
}

pub fn solve_manhattan(state: Cube) -> Vec<Move> {
    search_idastar(
        state,
        // // SAFETY: `manhattan_distance` returns a u8 which is always representable by a n f32
        // |cube| unsafe { NotNan::new_unchecked(manhattan_distance(cube) as f32) },
        manhattan_distance,
        Cube::is_solved,
    )
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

// // TODO: Move out to `search` module
// pub fn optimal_solution(self) -> Option<Vec<Move>> {
//     #[derive(Debug, Clone, Copy)]
//     struct Node {
//         cost: f32,
//         state: Cube,
//     }

//     impl Node {
//         fn neighbors(self) -> impl Iterator<Item = (Move, Self)> {
//             self.state.neighbors().map(move |(mov, next_state)| {
//                 (
//                     mov,
//                     Node {
//                         cost: self.cost + 1.0,
//                         state: next_state,
//                     },
//                 )
//             })
//         }
//     }

//     impl PartialEq for Node {
//         fn eq(&self, other: &Self) -> bool {
//             self.cost == other.cost
//         }
//     }

//     impl Eq for Node {}

//     impl PartialOrd for Node {
//         fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//             other.cost.partial_cmp(&self.cost)
//         }
//     }

//     impl Ord for Node {
//         fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//             other.cost.total_cmp(&self.cost)
//         }
//     }

//     let mut queue = BinaryHeap::new();
//     queue.push(Node {
//         cost: 0.0,
//         state: self,
//     });

//     // let mut visited = HashSet::new();
//     // let mut mov_for = HashMap::<_, Move>::new();

//     let path = pathfinding::directed::bfs::bfs(
//         &self,
//         |c| c.neighbors().map(|(_, n)| n),
//         |c| c.is_solved(),
//     )?;

//     let solution = path
//         .windows(2)
//         .map(|window| {
//             let start = window[0];
//             let end = window[1];

//             Move::connect(start, end).unwrap()
//         })
//         .collect::<Vec<_>>();

//     Some(solution)

//     // while let Some(node) = queue.pop() {
//     //     if node.state.is_solved() {
//     //         let mut solution = Vec::new();
//     //         let mut state = node.state;
//     //         while let Some(&mov) = mov_for.get(&state) {
//     //             dbg!(&solution);
//     //             solution.push(mov);
//     //             state = state.mov_single(mov.reverse())
//     //         }

//     //         return Some(solution);
//     //     }

//     //     if !visited.insert(node.state) {
//     //         continue;
//     //     }

//     //     for (mov, neighbor) in node.neighbors() {
//     //         if !visited.insert(neighbor.state) {
//     //             continue;
//     //         }

//     //         mov_for.insert(neighbor.state, mov);
//     //         queue.push(neighbor);
//     //     }
//     // }

//     // None
// }
