//! This implementation is based on https://qiita.com/7y2n/items/55abb991a45ade2afa28

use norcina_core::math::{choose, fac};
use norcina_cube_n::{
    math::Axis,
    mov::Move,
    piece::{
        corner::{self, Corner},
        edge::{self, Edge},
    },
};

use crate::{Cube, search::kociemba::is_in_g1};

use super::G1_MOVES;

/// Can be used as a heuristic. Stores, for each state, the minimum amount
/// of moves to:
/// - Given a scrambled cube...
///     - Orient the corners
///     - Orient the edges
///     - Put the y-slice edges to the y-slice
/// - Given a cube in G1...
///     - Permute the corners
///     - Permute the y-slice edges
///     - Permute the U/D-face edges
#[derive(Debug)]
pub struct PruneTable {
    orient_corners: Vec<u8>,
    orient_edges: Vec<u8>,
    put_edges_to_y_slice: Vec<u8>,
    permute_corners: Vec<u8>,
    permute_y_slice_edges: Vec<u8>,
    permute_non_y_slice_edges: Vec<u8>,
}

impl PruneTable {
    /// Loads the prune table from disk, or creates it if it doesn't exist yet.
    pub fn load_or_generate() -> Self {
        // TODO: Actually handle saving and loading from disk, but it just isn't
        // necessary for now because it takes just 40ms in my machine.
        Self::generate()
    }

    /// Constructs the prune table from scratch.
    // TODO: Write how long this takes to run on my machine.
    pub fn generate() -> Self {
        PruneTable {
            orient_corners: CORNER_ORIENTATION.generate_buffer(),
            orient_edges: EDGE_ORIENTATION.generate_buffer(),
            put_edges_to_y_slice: IS_ON_Y_SLICE.generate_buffer(),
            permute_corners: CORNER_POSITION.generate_buffer(),
            permute_y_slice_edges: Y_SLICE_POSITION.generate_buffer(),
            permute_non_y_slice_edges: NON_Y_SLICE_POSITION.generate_buffer(),
        }
    }

    pub fn phase1_distance_heuristic(&self, cube: Cube) -> u8 {
        let co = self.orient_corners[(CORNER_ORIENTATION.index)(cube.corners)];
        let eo = self.orient_edges[(EDGE_ORIENTATION.index)(cube.edges)];
        let y_slice = self.put_edges_to_y_slice[(IS_ON_Y_SLICE.index)(cube.edges)];

        co.max(eo).max(y_slice)
    }

    pub fn phase2_distance_heuristic(&self, cube: Cube) -> u8 {
        debug_assert!(is_in_g1(cube));
        let pc = self.permute_corners[(CORNER_POSITION.index)(cube.corners)];
        let pye = self.permute_y_slice_edges[(Y_SLICE_POSITION.index)(cube.edges)];
        let pnye = self.permute_non_y_slice_edges[(NON_Y_SLICE_POSITION.index)(cube.edges)];

        pc.max(pye).max(pnye)
    }
}

#[derive(Debug, Clone, Copy)]
struct Subtable<T> {
    index: fn(T) -> usize,
    from_index: fn(usize) -> T,
    max: usize,
    initial: T,
    phase1: bool,
    apply_mov: fn(T, Move) -> T,
}

impl<T: Copy + std::fmt::Debug> Subtable<T> {
    fn generate_buffer(self) -> Vec<u8> {
        let Self {
            index,
            from_index,
            max,
            initial,
            phase1,
            apply_mov,
        } = self;
        let moves = if phase1 {
            Move::ALL.as_slice()
        } else {
            G1_MOVES.as_slice()
        };

        let mut buffer = vec![u8::MAX; max];
        buffer[index(initial)] = 0;

        for depth in 1.. {
            let mut complete = true;

            for i in 0..max {
                if buffer[i] != depth - 1 {
                    continue;
                }

                let state = from_index(i);
                for mov in moves {
                    let new_state = apply_mov(state, *mov);
                    let new_index = index(new_state);

                    if buffer[new_index] > depth {
                        buffer[new_index] = depth;
                        complete = false;
                    }
                }
            }

            if complete {
                break;
            }
        }

        buffer
    }
}

// -- Phase 1 --

const CORNER_ORIENTATION: Subtable<[Corner; 8]> = Subtable {
    index: |corners| {
        let mut index = 0;
        for corner in &corners[0..7] {
            index *= 3;
            index += corner.orientation().u8() as usize;
        }

        index
    },
    from_index: |mut index| {
        let mut corners = Corner::SOLVED;
        let mut orientation_sum = 0;
        for corner in corners[0..7].iter_mut().rev() {
            let orientation = Axis::from_u8((index % 3) as u8);
            corner.set_orientation(orientation);
            index /= 3;
            orientation_sum += orientation.u8();
        }

        corners[7].set_orientation(Axis::from_i8_mod3(-(orientation_sum as i8)));
        corners
    },
    initial: Corner::SOLVED,
    max: 3usize.pow(8 - 1),
    phase1: true,
    apply_mov: corner::move_pieces,
};

const EDGE_ORIENTATION: Subtable<[Edge; 12]> = Subtable {
    index: |edges| {
        let mut output = 0;
        for edge in &edges[0..11] {
            output *= 2;
            output += edge.orientation() as usize;
        }
        output
    },
    from_index: |mut index| {
        let mut edges = Edge::SOLVED;
        let mut orientation_sum = 0;
        for edge in edges[0..11].iter_mut().rev() {
            let is_oriented = index % 2 == 0;
            edge.set_oriented(is_oriented);
            index /= 2;
            orientation_sum += is_oriented as u8;
        }

        edges[11].set_oriented(orientation_sum % 2 == 0);

        edges
    },
    initial: Edge::SOLVED,
    max: 2usize.pow(12 - 1),
    phase1: true,
    apply_mov: edge::move_pieces,
};

// TODO: This is just n choose r, right?
fn calc_combination(n: usize, r: usize) -> usize {
    let mut output = 1;
    // n * (n - 1) * (n - 2) * ... * (n - r + 1)
    for i in 0..r {
        output *= n - i;
    }

    // r * (r - 1) * (r - 2) * ... * 1
    for i in 0..r {
        output /= r - i;
    }

    output
}

const IS_ON_Y_SLICE: Subtable<[Edge; 12]> = Subtable {
    index: |edges| {
        let mut index = 0;
        let mut remaining = 4;
        debug_assert_eq!(
            edges.iter().filter(|edge| edge.normal() == Axis::Y).count(),
            4,
            "Edges are: {:?}",
            edges
        );
        for (i, edge) in edges.iter().enumerate().rev() {
            if edge.position().normal() == Axis::Y {
                index += calc_combination(i, remaining);
                remaining -= 1;
            }
        }

        index
    },
    from_index: |mut index| {
        // We actually totally ignore this value. Instead, we go through each
        // edge and add the first available edge either on or off the y-normal
        // slice, depening on the array index and how many y-normal edges are
        // left.
        let mut edges = Edge::SOLVED;
        let mut remaining = 4;

        for i in (0..12).rev() {
            if index >= calc_combination(i, remaining) {
                edges[i] = Edge::SOLVED[remaining + 3];
                index -= calc_combination(i, remaining);
                remaining -= 1;
            } else {
                edges[i] = Edge::SOLVED[(i + 8 - remaining) % 12];
            }
        }

        edges
    },
    initial: Edge::SOLVED,
    max: choose(12, 4),
    phase1: true,
    apply_mov: edge::move_pieces,
};

// -- Phase 2 --

const CORNER_POSITION: Subtable<[Corner; 8]> = Subtable {
    index: |corners| {
        let mut index = 0;
        for (i, c1) in corners.into_iter().enumerate() {
            index *= 8 - i;
            for c2 in &corners[i + 1..] {
                if c1.position().u8() > c2.position().u8() {
                    index += 1;
                }
            }
        }

        index
    },
    from_index: |mut index| {
        let mut corners = [0; 8];
        for i in (0..7).rev() {
            corners[i] = (index % (8 - i)) as u8;
            index /= 8 - i;
            for j in (i + 1)..8 {
                if corners[j] >= corners[i] {
                    corners[j] += 1;
                }
            }
        }

        // TODO: We could transmute, technically...
        corners.map(Corner::solved)
    },
    initial: Corner::SOLVED,
    max: fac(8),
    phase1: false,
    apply_mov: corner::move_pieces,
};

const Y_SLICE_POSITION: Subtable<[Edge; 12]> = Subtable {
    index: |edges| {
        let mut index = 0;
        // This is valid because we assume the cube is in G1.
        let edges = || edges[..4].iter().chain(&edges[8..]);

        for (i, e1) in edges().enumerate() {
            index *= 8 - i;
            for e2 in edges().skip(i + 1) {
                if e1.position().u8() > e2.position().u8() {
                    index += 1;
                }
            }
        }

        index
    },
    from_index: |mut index| {
        let mut edges = [0; 8];
        for i in (0..7).rev() {
            edges[i] = (index % (8 - i)) as u8;
            index /= 8 - i;
            for j in (i + 1)..8 {
                if edges[j] >= edges[i] {
                    edges[j] += 1;
                }
            }
        }

        let mut output = Edge::SOLVED;
        for i in 0..4 {
            output[i] = Edge::solved(edges[i]);
        }
        for i in 8..12 {
            output[i] = Edge::solved(edges[i - 4]);
        }

        output
    },
    initial: Edge::SOLVED,
    max: fac(8),
    phase1: false,
    apply_mov: edge::move_pieces,
};

const NON_Y_SLICE_POSITION: Subtable<[Edge; 12]> = Subtable {
    index: |edges| {
        let mut index = 0;
        // This is valid because we assume the cube is in G1.
        let edges = &edges[4..8];

        for (i, e1) in edges.iter().enumerate() {
            index *= 4 - i;
            for e2 in &edges[i + 1..] {
                if e1.position().u8() > e2.position().u8() {
                    index += 1;
                }
            }
        }

        index
    },
    from_index: |mut index| {
        let mut edges = [0; 4];
        for i in (0..3).rev() {
            edges[i] = (index % (4 - i)) as u8;
            index /= 4 - i;
            for j in (i + 1)..4 {
                if edges[j] >= edges[i] {
                    edges[j] += 1;
                }
            }
        }

        let mut output = Edge::SOLVED;
        for i in 0..4 {
            output[i + 4] = Edge::solved(edges[i]);
        }
        output
    },
    initial: Edge::SOLVED,
    max: fac(4),
    phase1: false,
    apply_mov: edge::move_pieces,
};

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;
    use std::fmt::Debug;

    fn test_id<T: Debug>(subtable: Subtable<T>, i: usize) -> bool {
        let Subtable {
            index,
            from_index,
            max,
            ..
        } = subtable;
        let i = i % max;
        i == index(from_index(i))
    }

    quickcheck! {
        fn fn_index_fn_from_index_is_identity_co(index: usize) -> bool { test_id(CORNER_ORIENTATION, index) }
        fn fn_index_fn_from_index_id_identity_eo(index: usize) -> bool { test_id(EDGE_ORIENTATION, index) }
        fn fn_index_fn_from_index_id_identity_yslice(index: usize) -> bool { test_id(IS_ON_Y_SLICE, index) }
        fn fn_index_fn_from_index_id_identity_pc(index: usize) -> bool { test_id(CORNER_POSITION, index) }
        fn fn_index_fn_from_index_id_identity_pye(index: usize) -> bool { test_id(Y_SLICE_POSITION, index) }
        fn fn_index_fn_from_index_id_identity_pnye(index: usize) -> bool { test_id(NON_Y_SLICE_POSITION, index) }
    }
}
