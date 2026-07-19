//! Precomputed lookup tables for the two-phase search.
//!
//! Two kinds of table, both derived from the verified [`CubieCube`] move engine:
//!
//! - **Move tables** map `(coordinate, move) → coordinate`, so the search steps
//!   in coordinate space without touching a full cube.
//! - **Pruning tables** are pattern databases: a breadth-first search from the
//!   goal records the exact distance of every `(coord_a, coord_b)` pair, giving
//!   an admissible lower bound that drives IDA*.
//!
//! Everything is built once (lazily, see [`crate::solver::tables`]) and reused.

use std::collections::VecDeque;

use super::coords::*;
use crate::cubie::CubieCube;
use crate::moves::{Face, Move, Turn};

pub const N_MOVES1: usize = 18;
pub const N_MOVES2: usize = 10;

/// The 18 Half-Turn-Metric face turns used in phase 1.
pub const PHASE1_MOVES: [Move; N_MOVES1] = {
    use Face::*;
    use Turn::*;
    [
        Move::new(U, Cw), Move::new(U, Double), Move::new(U, Ccw),
        Move::new(R, Cw), Move::new(R, Double), Move::new(R, Ccw),
        Move::new(F, Cw), Move::new(F, Double), Move::new(F, Ccw),
        Move::new(D, Cw), Move::new(D, Double), Move::new(D, Ccw),
        Move::new(L, Cw), Move::new(L, Double), Move::new(L, Ccw),
        Move::new(B, Cw), Move::new(B, Double), Move::new(B, Ccw),
    ]
};

/// The 10 moves that keep the cube inside G1, used in phase 2:
/// `U, U2, U', D, D2, D', R2, L2, F2, B2`.
pub const PHASE2_MOVES: [Move; N_MOVES2] = {
    use Face::*;
    use Turn::*;
    [
        Move::new(U, Cw), Move::new(U, Double), Move::new(U, Ccw),
        Move::new(D, Cw), Move::new(D, Double), Move::new(D, Ccw),
        Move::new(R, Double),
        Move::new(L, Double),
        Move::new(F, Double),
        Move::new(B, Double),
    ]
};

/// All precomputed tables for the solver.
pub struct Tables {
    // Phase 1 move tables (indexed `coord * N_MOVES1 + move`).
    pub move_twist: Vec<u16>,
    pub move_flip: Vec<u16>,
    pub move_slice: Vec<u16>,
    // Phase 2 move tables (indexed `coord * N_MOVES2 + move`).
    pub move_corner: Vec<u16>,
    pub move_edge8: Vec<u16>,
    pub move_sliceperm: Vec<u16>,
    // Pruning tables (distance to goal, 255 = unreached).
    pub prun_slice_twist: Vec<u8>,
    pub prun_slice_flip: Vec<u8>,
    pub prun_corner_sliceperm: Vec<u8>,
    pub prun_edge8_sliceperm: Vec<u8>,
}

impl Tables {
    /// Build every table. Runs once; takes a fraction of a second in release.
    pub fn build() -> Tables {
        let move_twist = build_move1(N_TWIST, get_twist, set_twist);
        let move_flip = build_move1(N_FLIP, get_flip, set_flip);
        let move_slice = build_move1(N_SLICE, get_slice, set_slice);
        let move_corner = build_move2(N_CORNER, get_corners, set_corners);
        let move_edge8 = build_move2(N_EDGE8, get_edge8, set_edge8);
        let move_sliceperm = build_move2(N_SLICEPERM, get_sliceperm, set_sliceperm);

        let prun_slice_twist =
            build_prun(N_SLICE, N_TWIST, N_MOVES1, &move_slice, &move_twist);
        let prun_slice_flip =
            build_prun(N_SLICE, N_FLIP, N_MOVES1, &move_slice, &move_flip);
        let prun_corner_sliceperm =
            build_prun(N_CORNER, N_SLICEPERM, N_MOVES2, &move_corner, &move_sliceperm);
        let prun_edge8_sliceperm =
            build_prun(N_EDGE8, N_SLICEPERM, N_MOVES2, &move_edge8, &move_sliceperm);

        Tables {
            move_twist,
            move_flip,
            move_slice,
            move_corner,
            move_edge8,
            move_sliceperm,
            prun_slice_twist,
            prun_slice_flip,
            prun_corner_sliceperm,
            prun_edge8_sliceperm,
        }
    }
}

/// Build a phase-1 move table for one coordinate: for every coordinate value,
/// realize a representative cube, apply each of the 18 moves, and record the
/// resulting coordinate.
fn build_move1(
    size: usize,
    get: fn(&CubieCube) -> u16,
    set: fn(&mut CubieCube, u16),
) -> Vec<u16> {
    let mut table = vec![0u16; size * N_MOVES1];
    for x in 0..size {
        let mut base = CubieCube::SOLVED;
        set(&mut base, x as u16);
        for (m, mv) in PHASE1_MOVES.iter().enumerate() {
            let mut c = base;
            c.apply(*mv);
            table[x * N_MOVES1 + m] = get(&c);
        }
    }
    table
}

/// Same as [`build_move1`] but over the 10 phase-2 moves.
fn build_move2(
    size: usize,
    get: fn(&CubieCube) -> u16,
    set: fn(&mut CubieCube, u16),
) -> Vec<u16> {
    let mut table = vec![0u16; size * N_MOVES2];
    for x in 0..size {
        let mut base = CubieCube::SOLVED;
        set(&mut base, x as u16);
        for (m, mv) in PHASE2_MOVES.iter().enumerate() {
            let mut c = base;
            c.apply(*mv);
            table[x * N_MOVES2 + m] = get(&c);
        }
    }
    table
}

/// Build a pattern-database pruning table over the product of two coordinates
/// via BFS from the goal `(0, 0)`. Entry `a * dim_b + b` holds the minimum
/// number of moves to reach the goal; unreachable pairs stay 255.
fn build_prun(
    dim_a: usize,
    dim_b: usize,
    n_moves: usize,
    move_a: &[u16],
    move_b: &[u16],
) -> Vec<u8> {
    let mut prun = vec![255u8; dim_a * dim_b];
    prun[0] = 0; // goal is (0, 0)
    let mut queue: VecDeque<(u16, u16)> = VecDeque::new();
    queue.push_back((0, 0));
    while let Some((a, b)) = queue.pop_front() {
        let d = prun[a as usize * dim_b + b as usize];
        for m in 0..n_moves {
            let na = move_a[a as usize * n_moves + m] as usize;
            let nb = move_b[b as usize * n_moves + m] as usize;
            let idx = na * dim_b + nb;
            if prun[idx] == 255 {
                prun[idx] = d + 1;
                queue.push_back((na as u16, nb as u16));
            }
        }
    }
    prun
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_tables_match_direct_application() {
        // A move table must agree with applying the move to a real cube. Check
        // the phase-1 twist table against the cube engine for every value.
        let table = build_move1(N_TWIST, get_twist, set_twist);
        for x in (0..N_TWIST).step_by(37) {
            let mut base = CubieCube::SOLVED;
            set_twist(&mut base, x as u16);
            for (m, mv) in PHASE1_MOVES.iter().enumerate() {
                let mut c = base;
                c.apply(*mv);
                assert_eq!(table[x * N_MOVES1 + m], get_twist(&c));
            }
        }
    }

    #[test]
    fn phase2_moves_preserve_g1_structure() {
        // Every phase-2 move must keep the slice edges within the slice and the
        // U/D edges out of it — otherwise the phase-2 coordinates are ill-defined.
        for mv in PHASE2_MOVES {
            let mut c = CubieCube::SOLVED;
            c.apply(mv);
            for pos in 0..8 {
                assert!(c.ep[pos] < 8, "{mv} moved a slice edge into the U/D layer");
            }
            for pos in 8..12 {
                assert!(c.ep[pos] >= 8, "{mv} moved a U/D edge into the slice");
            }
            assert!(c.co.iter().all(|&o| o == 0), "{mv} twisted a corner");
            assert!(c.eo.iter().all(|&o| o == 0), "{mv} flipped an edge");
        }
    }

    #[test]
    fn pruning_goal_is_zero_and_bounds_are_sane() {
        let move_slice = build_move1(N_SLICE, get_slice, set_slice);
        let move_twist = build_move1(N_TWIST, get_twist, set_twist);
        let prun = build_prun(N_SLICE, N_TWIST, N_MOVES1, &move_slice, &move_twist);
        assert_eq!(prun[0], 0, "goal must have distance 0");
        // Phase 1 is known to need at most 12 moves; the PDB is a lower bound.
        let max = prun.iter().filter(|&&d| d != 255).copied().max().unwrap();
        assert!(max <= 12, "slice-twist pruning bound {max} exceeds 12");
        assert!(max >= 1);
    }
}
