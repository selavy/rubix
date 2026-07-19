//! Facelet (sticker) view of the cube.
//!
//! The [`CubieCube`](crate::CubieCube) coordinate model is the source of truth;
//! this module derives the 54-sticker view from it on demand, for the renderer
//! and for orientation-independent win detection. Sticker layout matches the
//! standard Kociemba facelet numbering:
//!
//! ```text
//! indices 0..53, nine per face, in face order U R F D L B:
//!   U: 0..8   R: 9..17   F: 18..26   D: 27..35   L: 36..44   B: 45..53
//! within a face:  1 2 3 / 4 5 6 / 7 8 9  (row-major)
//! ```

use crate::CubieCube;

/// A sticker color, one per face. The renderer maps these logical colors to
/// concrete RGB (the standard Western scheme: U=white, D=yellow, F=green,
/// B=blue, R=red, L=orange). Discriminants match [`crate::Face`] indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    U = 0,
    R = 1,
    F = 2,
    D = 3,
    L = 4,
    B = 5,
}

impl Color {
    const ALL: [Color; 6] = [Color::U, Color::R, Color::F, Color::D, Color::L, Color::B];
}

// The three facelet indices making up each corner cubie slot, ordered so that
// index 0 is the sticker twisted "into place" by orientation 0. Corner order:
// URF, UFL, ULB, UBR, DFR, DLF, DBL, DRB.
const CORNER_FACELET: [[u8; 3]; 8] = [
    [8, 9, 20],   // URF: U9 R1 F3
    [6, 18, 38],  // UFL: U7 F1 L3
    [0, 36, 47],  // ULB: U1 L1 B3
    [2, 45, 11],  // UBR: U3 B1 R3
    [29, 26, 15], // DFR: D3 F9 R7
    [27, 44, 24], // DLF: D1 L9 F7
    [33, 53, 42], // DBL: D7 B9 L7
    [35, 17, 51], // DRB: D9 R9 B7
];

// The colors of each corner cubie's three stickers, in the same sticker order.
const CORNER_COLOR: [[Color; 3]; 8] = [
    [Color::U, Color::R, Color::F], // URF
    [Color::U, Color::F, Color::L], // UFL
    [Color::U, Color::L, Color::B], // ULB
    [Color::U, Color::B, Color::R], // UBR
    [Color::D, Color::F, Color::R], // DFR
    [Color::D, Color::L, Color::F], // DLF
    [Color::D, Color::B, Color::L], // DBL
    [Color::D, Color::R, Color::B], // DRB
];

// The two facelet indices making up each edge slot. Edge order:
// UR, UF, UL, UB, DR, DF, DL, DB, FR, FL, BL, BR.
const EDGE_FACELET: [[u8; 2]; 12] = [
    [5, 10],  // UR: U6 R2
    [7, 19],  // UF: U8 F2
    [3, 37],  // UL: U4 L2
    [1, 46],  // UB: U2 B2
    [32, 16], // DR: D6 R8
    [28, 25], // DF: D2 F8
    [30, 43], // DL: D4 L8
    [34, 52], // DB: D8 B8
    [23, 12], // FR: F6 R4
    [21, 41], // FL: F4 L6
    [50, 39], // BL: B6 L4
    [48, 14], // BR: B4 R6
];

// The colors of each edge cubie's two stickers, in the same sticker order.
const EDGE_COLOR: [[Color; 2]; 12] = [
    [Color::U, Color::R], // UR
    [Color::U, Color::F], // UF
    [Color::U, Color::L], // UL
    [Color::U, Color::B], // UB
    [Color::D, Color::R], // DR
    [Color::D, Color::F], // DF
    [Color::D, Color::L], // DL
    [Color::D, Color::B], // DB
    [Color::F, Color::R], // FR
    [Color::F, Color::L], // FL
    [Color::B, Color::L], // BL
    [Color::B, Color::R], // BR
];

// Center sticker index for each face (fixed). Order U R F D L B.
const CENTER_FACELET: [u8; 6] = [4, 13, 22, 31, 40, 49];

/// The 54 sticker colors of `cube`, in canonical facelet order.
pub fn to_facelets(cube: &CubieCube) -> [Color; 54] {
    // Centers are fixed; seed with them so every index is initialized.
    let mut f = [Color::U; 54];
    for (face, &idx) in CENTER_FACELET.iter().enumerate() {
        f[idx as usize] = Color::ALL[face];
    }
    for i in 0..8 {
        let cubie = cube.cp[i] as usize;
        let ori = cube.co[i] as usize;
        for k in 0..3 {
            let slot = CORNER_FACELET[i][(k + ori) % 3] as usize;
            f[slot] = CORNER_COLOR[cubie][k];
        }
    }
    for i in 0..12 {
        let cubie = cube.ep[i] as usize;
        let ori = cube.eo[i] as usize;
        for k in 0..2 {
            let slot = EDGE_FACELET[i][(k + ori) % 2] as usize;
            f[slot] = EDGE_COLOR[cubie][k];
        }
    }
    f
}

/// True when every face shows a single uniform color. This is the win
/// condition: it holds for the solved cube in *any* orientation (see
/// `DESIGN.md` §4.5).
pub fn is_solved(cube: &CubieCube) -> bool {
    let f = to_facelets(cube);
    f.chunks_exact(9).all(|face| face.iter().all(|&c| c == face[0]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solved_facelets_are_uniform_per_face() {
        let f = to_facelets(&CubieCube::SOLVED);
        // Each 9-sticker face equals its center color, in face order U R F D L B.
        for (face, chunk) in f.chunks_exact(9).enumerate() {
            let expected = Color::ALL[face];
            assert!(chunk.iter().all(|&c| c == expected), "face {face} not uniform");
        }
    }

    #[test]
    fn solved_cube_is_solved() {
        assert!(is_solved(&CubieCube::SOLVED));
    }

    #[test]
    fn every_facelet_index_is_written_exactly_once() {
        // Sanity check on the lookup tables: the corner/edge/center facelet
        // indices must together cover 0..54 with no repeats.
        let mut seen = [0u8; 54];
        for c in CORNER_FACELET.iter() {
            for &i in c {
                seen[i as usize] += 1;
            }
        }
        for e in EDGE_FACELET.iter() {
            for &i in e {
                seen[i as usize] += 1;
            }
        }
        for &i in CENTER_FACELET.iter() {
            seen[i as usize] += 1;
        }
        assert!(seen.iter().all(|&n| n == 1), "facelet index coverage: {seen:?}");
    }
}
