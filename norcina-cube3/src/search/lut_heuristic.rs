use color_eyre::eyre::{self, WrapErr as _};
use norcina_core::math::{comb, fac};
use std::{fs, io, path::PathBuf};

use crate::{
    Cube, Move,
    math::{Axis, Direction},
    piece::{
        corner::{self, Corner, CornerPosition},
        edge::{Edge, EdgePosition},
    },
};

pub struct TableHeuristic {
    edges: Vec<u8>,
    corners: Vec<u8>,
}

impl TableHeuristic {
    pub fn read() -> eyre::Result<Self> {
        let (corners_file, edges_file) = Self::paths()?;

        let msg = "Maybe run `norcina --generate-heuristic-cache` first";
        Ok(TableHeuristic {
            corners: fs::read(corners_file).wrap_err(msg)?,
            edges: fs::read(edges_file).wrap_err(msg)?,
        })
    }

    pub fn generate() -> io::Result<Self> {
        let (corners_file, edges_file) = Self::paths()?;

        let mut corner_cache = vec![u8::MAX; CORNER_STATES as usize];
        let mut edge_cache = vec![u8::MAX; HALF_EDGE_STATES as usize];

        corner_cache[index_corners(Cube::SOLVED.corners) as usize] = 0;
        edge_cache[index_edges(Cube::SOLVED.edges).0 as usize] = 0;
        edge_cache[index_edges(Cube::SOLVED.edges).1 as usize] = 0;

        // TODO: Make parallel
        for depth in 0.. {
            println!("Caching at depth={depth}");
            let mut remaining = false;
            for i in 0..CORNER_STATES {
                let v = corner_cache[i as usize];
                if v > depth as u8 {
                    remaining = true;
                    continue;
                } else if v < (depth as u8) {
                    continue;
                }

                let corners = corners_from_index(i);
                for mov in Move::iter() {
                    let neighbor = corner::move_pieces(corners, mov);
                    let neighbor_index = index_corners(neighbor);
                    let prev = corner_cache[neighbor_index as usize];
                    corner_cache[neighbor_index as usize] = prev.min(depth as u8 + 1);
                }
            }

            if !remaining {
                break;
            }
        }

        todo!("Populate edges");

        fs::write(corners_file, &corner_cache)?;
        fs::write(edges_file, &edge_cache)?;

        Ok(Self {
            corners: corner_cache,
            edges: edge_cache,
        })
    }

    fn create_paths() -> io::Result<(PathBuf, PathBuf)> {
        let (cpath, epath) = Self::paths()?;
        assert_eq!(cpath.parent(), epath.parent());
        fs::create_dir_all(cpath.parent().unwrap())?;

        Ok((cpath, epath))
    }

    fn paths() -> io::Result<(PathBuf, PathBuf)> {
        let mut cache_dir = dirs::cache_dir().ok_or(io::Error::other("No cache dir available."))?;
        cache_dir.push("norcina");

        let mut corners_file = cache_dir.clone();
        corners_file.push("corners.norcina");
        let mut edges_file = cache_dir;
        edges_file.push("edges.norcina");

        Ok((corners_file, edges_file))
    }
}

const CORNER_PERMUTATIONS: u32 = fac(8);
const CORNER_ORIENTATIONS: u32 = 3u32.pow(7);
const CORNER_STATES: u32 = CORNER_ORIENTATIONS * CORNER_PERMUTATIONS;

/// Returns a unique number between 0 and 88_179_839 for each possible corner set.
fn index_corners(corners: [Corner; 8]) -> u32 {
    let permutation_index: u32 = {
        let mut used_slots = [false; 8];
        let mut modulo = 1;
        let mut out = 0;
        for (i, &corner) in corners.iter().enumerate() {
            let index = corner.position().u8();
            let chosen_before = used_slots
                .iter()
                .take(index as usize)
                .filter(|&&u| u)
                .count() as u8;

            let choice = index - chosen_before;
            out += choice as u32 * modulo as u32;
            let number_of_choices = 8 - i;
            modulo *= number_of_choices;
            debug_assert!(used_slots[index as usize] == false);
            used_slots[index as usize] = true;
        }

        out
    };

    debug_assert!(permutation_index < CORNER_PERMUTATIONS);

    let orientation_index: u32 = corners
        .iter()
        .take(7)
        .enumerate()
        .map(|(i, corner)| 3u32.pow(i as u32) * corner.orientation().u8() as u32)
        .sum();

    debug_assert!(orientation_index < CORNER_ORIENTATIONS);

    // permutation_index + orientation_index * CORNER_PERMUTATIONS
    orientation_index + CORNER_ORIENTATIONS * permutation_index
}

fn indices_from_permutation_index<const N_CHOICES: usize, const N_TOTAL_OPTIONS: usize>(
    permutation_index: u32,
) -> impl Iterator<Item = u8> {
    let mut modulo = 1;
    let choices = (0..N_CHOICES as u32).map(move |i| {
        let number_of_choices = N_TOTAL_OPTIONS as u32 - i;
        let choice = (permutation_index / modulo) % number_of_choices;
        modulo *= number_of_choices;
        choice as u8
    });

    // Get index from choices
    let mut chosen_list = [false; N_TOTAL_OPTIONS];
    choices.map(move |choice| {
        let mut unchosen_index = 0;
        for (chosen_index, chosen) in chosen_list.into_iter().enumerate() {
            if chosen {
                continue;
            }

            if unchosen_index == choice {
                assert!(!chosen_list[chosen_index as usize]);
                chosen_list[chosen_index as usize] = true;
                return chosen_index as u8;
            }

            unchosen_index += 1;
        }

        unreachable!()
    })
}

fn corners_from_index(index: u32) -> [Corner; 8] {
    let permutation_index = index / CORNER_ORIENTATIONS;
    let orientation_index = index % CORNER_ORIENTATIONS;

    let mut out = Corner::SOLVED;
    let mut orientation_sum = 0u8;
    for (i, permutation_index) in
        indices_from_permutation_index::<8, 8>(permutation_index).enumerate()
    {
        let orientation = if i < 7 {
            Axis::from_u8_mod3(((orientation_index / 3u32.pow(i as u32)) % 3) as u8)
        } else {
            Axis::from_u8_mod3(3 - (orientation_sum % 3))
        };

        orientation_sum += orientation.u8();
        orientation.u8();
        out[i] = CornerPosition::from_index(permutation_index).with_orientation(orientation);
    }

    out
}

const HALF_EDGE_PERMUTATIONS: u32 = comb(12, 6);
const HALF_EDGE_ORIENTATIONS: u32 = 2u32.pow(6);
const HALF_EDGE_STATES: u32 = HALF_EDGE_PERMUTATIONS * HALF_EDGE_ORIENTATIONS;

/// For each set of edges, returns a unique combination of numbers, where each one
/// is between 0 and 42_577_919
fn index_edges(edges: [Edge; 12]) -> (u32, u32) {
    let compute = |half| {
        // Take only half of the edges.
        // Probably can be more efficient, but this isn't a hot loop.
        let edges = edges.iter().skip(if half { 6 } else { 0 }).take(6);

        let orientation_index: u32 = edges
            .clone()
            .enumerate()
            .map(|(i, edge)| 2u32.pow(i as u32) * edge.orientation().u8() as u32)
            .sum();

        let permutation_index: u32 = {
            let mut used_slots = [false; 12];
            let mut modulo = 1;
            let mut out = 0;
            for (i, edge) in edges.enumerate() {
                let index = edge.position().index();
                let chosen_before = used_slots
                    .iter()
                    .take(index as usize)
                    .filter(|&&u| u)
                    .count() as u8;

                let choice = index - chosen_before;
                out += choice as u32 * modulo as u32;
                let number_of_choices = 12 - i;
                modulo *= number_of_choices;
                debug_assert!(used_slots[index as usize] == false);
                used_slots[index as usize] = true;
            }

            out
        };

        orientation_index + HALF_EDGE_ORIENTATIONS * permutation_index
    };

    (compute(false), compute(true))
}

fn edges_from_index((a, b): (u32, u32)) -> [Edge; 12] {
    let a_perm_i = a / HALF_EDGE_ORIENTATIONS;
    let a_orien_i = a % HALF_EDGE_ORIENTATIONS;
    let b_perm_i = b / HALF_EDGE_ORIENTATIONS;
    let b_orien_i = b % HALF_EDGE_ORIENTATIONS;

    let mut out = Edge::SOLVED;

    let mut populate = |perm_i, orien_i: u32, start| {
        let mut final_orientation = Direction::Positive;

        for (i, permutation_index) in indices_from_permutation_index::<6, 12>(perm_i).enumerate() {
            let orientation = if i < 11 {
                // Equivalent, but maybe more confusing? idk
                // Direction::from_u8_any((orien_i >> i) as u8 & 0b1)
                Direction::from_bool(orien_i / 2u32.pow(i as u32) % 2 != 0)
            } else {
                final_orientation
            };

            final_orientation = final_orientation ^ orientation;
            out[i + start] =
                EdgePosition::from_index(permutation_index).with_orientation(orientation);
        }
    };

    populate(a_perm_i, a_orien_i, 0);
    populate(b_perm_i, b_orien_i, 6);

    out
}

fn half_edges_from_index(index: u32) -> [Edge; 6] {
    todo!()
}

#[cfg(test)]
mod tests {
    use quickcheck::quickcheck;

    use super::*;
    use crate::Cube;

    quickcheck! {
        fn corner_index_is_between_range(cube: Cube) -> bool {
            index_corners(cube.corners) < CORNER_STATES
        }

        fn edge_index_is_between_range(cube: Cube) -> bool {
            let (a, b) = index_edges(cube.edges);
            a < HALF_EDGE_STATES && b < HALF_EDGE_STATES
        }

        fn index_from_index_identity(cube: Cube) -> bool {
            corners_from_index(index_corners(cube.corners)) == cube.corners &&
            edges_from_index(index_edges(cube.edges)) == cube.edges
        }
    }

    #[test]
    fn solved_cube_index_is_0() {
        assert_eq!(index_corners(Cube::SOLVED.corners), 0);
        assert_eq!(index_edges(Cube::SOLVED.edges).0, 0);
    }

    #[test]
    fn all_corner_indices_unique() {
        for i in 0..CORNER_STATES {
            assert_eq!(i, index_corners(corners_from_index(i)))
        }
    }

    #[test]
    fn all_edges_indices_unique() {
        for i in 0..HALF_EDGE_STATES {
            for j in 0..HALF_EDGE_STATES {
                assert_eq!((i, j), index_edges(edges_from_index((i, j))));
            }
        }
    }
}
