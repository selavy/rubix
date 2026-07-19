//! The two-phase IDA* search.
//!
//! Phase 1 drives the cube into G1 (twist = flip = slice = 0). Each time it
//! reaches G1 it hands the resulting cube to phase 2, which solves it within G1
//! (corner = edge8 = sliceperm = 0). We return the first complete solution
//! found — the shortest phase-1 maneuver, completed by phase 2 (see
//! `DESIGN.md` §5: "first solution", a short but not provably minimal answer).

use super::coords::*;
use super::tables::{Tables, N_MOVES1, N_MOVES2, PHASE1_MOVES, PHASE2_MOVES};
use crate::cubie::CubieCube;
use crate::moves::{Face, Move, Turn};

/// Any cube reaches G1 in at most 12 moves.
const MAX_PHASE1: u8 = 12;
/// Any G1 cube is solved by phase 2 in at most 18 moves.
const MAX_PHASE2: u8 = 18;

/// Find a solution for `cube`, or `None` if it is somehow unsolvable (never
/// happens for a legally reachable cube). An already-solved cube yields `[]`.
pub fn solve(tables: &Tables, cube: &CubieCube) -> Option<Vec<Move>> {
    let twist = get_twist(cube);
    let flip = get_flip(cube);
    let slice = get_slice(cube);
    let mut path = Vec::new();
    let mut out = None;
    for bound in 0..=MAX_PHASE1 {
        if search1(tables, cube, twist, flip, slice, 0, bound, None, &mut path, &mut out) {
            return out;
        }
        path.clear();
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn search1(
    t: &Tables,
    orig: &CubieCube,
    twist: u16,
    flip: u16,
    slice: u16,
    g: u8,
    bound: u8,
    last: Option<Face>,
    path: &mut Vec<u8>,
    out: &mut Option<Vec<Move>>,
) -> bool {
    if twist == 0 && flip == 0 && slice == 0 {
        // Reached G1. Rebuild the actual cube and solve it within G1.
        let mut cube = *orig;
        for &mi in path.iter() {
            cube.apply(PHASE1_MOVES[mi as usize]);
        }
        if let Some(p2) = solve_phase2(t, &cube) {
            let mut moves: Vec<Move> =
                path.iter().map(|&mi| PHASE1_MOVES[mi as usize]).collect();
            moves.extend(p2.iter().map(|&mi| PHASE2_MOVES[mi as usize]));
            *out = Some(simplify(&moves));
            return true;
        }
        return false;
    }
    if g == bound {
        return false;
    }
    let h = t.prun_slice_twist[slice as usize * N_TWIST + twist as usize]
        .max(t.prun_slice_flip[slice as usize * N_FLIP + flip as usize]);
    if g + h > bound {
        return false;
    }
    for m in 0..N_MOVES1 {
        let face = PHASE1_MOVES[m].face;
        if !allowed(last, face) {
            continue;
        }
        let nt = t.move_twist[twist as usize * N_MOVES1 + m];
        let nf = t.move_flip[flip as usize * N_MOVES1 + m];
        let ns = t.move_slice[slice as usize * N_MOVES1 + m];
        path.push(m as u8);
        if search1(t, orig, nt, nf, ns, g + 1, bound, Some(face), path, out) {
            return true;
        }
        path.pop();
    }
    false
}

fn solve_phase2(t: &Tables, cube: &CubieCube) -> Option<Vec<u8>> {
    let corner = get_corners(cube);
    let edge8 = get_edge8(cube);
    let sp = get_sliceperm(cube);
    let mut path = Vec::new();
    for bound in 0..=MAX_PHASE2 {
        if search2(t, corner, edge8, sp, 0, bound, None, &mut path) {
            return Some(path);
        }
        path.clear();
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn search2(
    t: &Tables,
    corner: u16,
    edge8: u16,
    sp: u16,
    g: u8,
    bound: u8,
    last: Option<Face>,
    path: &mut Vec<u8>,
) -> bool {
    if corner == 0 && edge8 == 0 && sp == 0 {
        return true;
    }
    if g == bound {
        return false;
    }
    let h = t.prun_corner_sliceperm[corner as usize * N_SLICEPERM + sp as usize]
        .max(t.prun_edge8_sliceperm[edge8 as usize * N_SLICEPERM + sp as usize]);
    if g + h > bound {
        return false;
    }
    for m in 0..N_MOVES2 {
        let face = PHASE2_MOVES[m].face;
        if !allowed(last, face) {
            continue;
        }
        let nc = t.move_corner[corner as usize * N_MOVES2 + m];
        let ne = t.move_edge8[edge8 as usize * N_MOVES2 + m];
        let nsp = t.move_sliceperm[sp as usize * N_MOVES2 + m];
        path.push(m as u8);
        if search2(t, nc, ne, nsp, g + 1, bound, Some(face), path) {
            return true;
        }
        path.pop();
    }
    false
}

/// Whether two faces are on the same axis (opposite faces of the cube).
fn opposite(a: Face, b: Face) -> bool {
    let (x, y) = (a as u8, b as u8);
    x.max(y) - x.min(y) == 3
}

/// Move-ordering filter that prunes redundant sequences: never turn the same
/// face twice in a row, and for a pair of opposite faces only allow the
/// canonical order (U before D, R before L, F before B), since they commute.
fn allowed(prev: Option<Face>, cur: Face) -> bool {
    match prev {
        None => true,
        Some(p) => p != cur && !(opposite(p, cur) && (cur as u8) < (p as u8)),
    }
}

/// Merge adjacent same-face turns and drop cancellations, so the seam between
/// the two phases reads cleanly (e.g. `U U2` → `U'`, `R R'` → nothing).
fn simplify(moves: &[Move]) -> Vec<Move> {
    let mut out: Vec<Move> = Vec::with_capacity(moves.len());
    for &m in moves {
        if let Some(&last) = out.last() {
            if last.face == m.face {
                out.pop();
                let combined = (last.turn.quarter_turns() + m.turn.quarter_turns()) % 4;
                match combined {
                    1 => out.push(Move::new(m.face, Turn::Cw)),
                    2 => out.push(Move::new(m.face, Turn::Double)),
                    3 => out.push(Move::new(m.face, Turn::Ccw)),
                    _ => {} // 0: the two turns cancel
                }
                continue;
            }
        }
        out.push(m);
    }
    out
}
