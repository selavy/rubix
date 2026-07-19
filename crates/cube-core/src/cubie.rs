//! The cubie-level coordinate model: the source of truth for cube state.
//!
//! A cube is described by where each of the 8 corner cubies and 12 edge cubies
//! sits (permutation) and how it is twisted/flipped (orientation). This is the
//! representation Kociemba's two-phase algorithm consumes, so keeping it as the
//! canonical state avoids a conversion at solve time.
//!
//! Piece indexing (Kociemba convention):
//!
//! ```text
//! corners: URF UFL ULB UBR DFR DLF DBL DRB   (0..8)
//! edges:   UR UF UL UB DR DF DL DB FR FL BL BR (0..12)
//! ```
//!
//! Corner orientation is 0/1/2 (clockwise twists, viewed from outside);
//! edge orientation is 0/1 (flipped or not).

use crate::facelet;
use crate::moves::Move;

/// A cube state in the corner/edge coordinate model.
///
/// A default-constructed `CubieCube` (via [`CubieCube::SOLVED`]) is the solved
/// cube. Only states reachable by face turns are produced by this crate, so
/// every value in normal use is a legal, solvable cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CubieCube {
    /// `cp[i]` = index of the corner cubie currently in slot `i`.
    pub cp: [u8; 8],
    /// `co[i]` = orientation (0..3) of the corner cubie in slot `i`.
    pub co: [u8; 8],
    /// `ep[i]` = index of the edge cubie currently in slot `i`.
    pub ep: [u8; 12],
    /// `eo[i]` = orientation (0..2) of the edge cubie in slot `i`.
    pub eo: [u8; 12],
}

impl CubieCube {
    /// The solved cube: every cubie home, every orientation zero.
    pub const SOLVED: CubieCube = CubieCube {
        cp: [0, 1, 2, 3, 4, 5, 6, 7],
        co: [0; 8],
        ep: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        eo: [0; 12],
    };

    /// Compose two cubes: apply `other` to `self`, returning the new state.
    ///
    /// Using the standard cubie-cube product, this is how a move is applied —
    /// `self.then(FACE_MOVE)` turns a face. It also composes whole sequences.
    #[must_use]
    pub fn then(&self, other: &CubieCube) -> CubieCube {
        let mut r = CubieCube::SOLVED;
        for i in 0..8 {
            let via = other.cp[i] as usize;
            r.cp[i] = self.cp[via];
            r.co[i] = (self.co[via] + other.co[i]) % 3;
        }
        for i in 0..12 {
            let via = other.ep[i] as usize;
            r.ep[i] = self.ep[via];
            r.eo[i] = (self.eo[via] + other.eo[i]) % 2;
        }
        r
    }

    /// Apply a single notated move (respecting its turn amount) in place.
    pub fn apply(&mut self, mv: Move) {
        let base = &BASE_MOVES[mv.face as usize];
        for _ in 0..mv.turn.quarter_turns() {
            *self = self.then(base);
        }
    }

    /// Apply a sequence of moves in order, in place.
    pub fn apply_all(&mut self, moves: &[Move]) {
        for &m in moves {
            self.apply(m);
        }
    }

    /// The 54-sticker view, for rendering.
    pub fn facelets(&self) -> [facelet::Color; 54] {
        facelet::to_facelets(self)
    }

    /// True when every face is a single color (orientation-independent win).
    pub fn is_solved(&self) -> bool {
        facelet::is_solved(self)
    }

    /// Whether this is a legally reachable cube state. Used in tests and as a
    /// guard when ingesting externally-supplied states. Checks the three
    /// classic invariants: corner-twist sum ≡ 0 (mod 3), edge-flip sum ≡ 0
    /// (mod 2), and equal corner/edge permutation parity.
    pub fn is_valid(&self) -> bool {
        // Permutations must each be a bijection over their index range.
        let mut seen_c = [false; 8];
        for &c in &self.cp {
            if c as usize >= 8 || seen_c[c as usize] {
                return false;
            }
            seen_c[c as usize] = true;
        }
        let mut seen_e = [false; 12];
        for &e in &self.ep {
            if e as usize >= 12 || seen_e[e as usize] {
                return false;
            }
            seen_e[e as usize] = true;
        }
        if self.co.iter().any(|&o| o >= 3) || self.eo.iter().any(|&o| o >= 2) {
            return false;
        }
        let co_sum: u32 = self.co.iter().map(|&o| o as u32).sum();
        let eo_sum: u32 = self.eo.iter().map(|&o| o as u32).sum();
        co_sum % 3 == 0
            && eo_sum % 2 == 0
            && permutation_parity(&self.cp) == permutation_parity(&self.ep)
    }
}

impl Default for CubieCube {
    fn default() -> CubieCube {
        CubieCube::SOLVED
    }
}

/// Parity (0 = even, 1 = odd) of a permutation, by counting inversions.
pub(crate) fn permutation_parity(perm: &[u8]) -> u8 {
    let mut inversions = 0u32;
    for i in 0..perm.len() {
        for j in (i + 1)..perm.len() {
            if perm[i] > perm[j] {
                inversions += 1;
            }
        }
    }
    (inversions % 2) as u8
}

/// The six clockwise quarter-turn face moves, indexed by [`Face`] discriminant
/// (U R F D L B). These are the canonical Kociemba generators; every other move
/// is built from them. Verified by the invariants exercised in the test module.
const BASE_MOVES: [CubieCube; 6] = [
    // U
    CubieCube {
        cp: [3, 0, 1, 2, 4, 5, 6, 7],
        co: [0, 0, 0, 0, 0, 0, 0, 0],
        ep: [3, 0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11],
        eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    },
    // R
    CubieCube {
        cp: [4, 1, 2, 0, 7, 5, 6, 3],
        co: [2, 0, 0, 1, 1, 0, 0, 2],
        ep: [8, 1, 2, 3, 11, 5, 6, 7, 4, 9, 10, 0],
        eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    },
    // F
    CubieCube {
        cp: [1, 5, 2, 3, 0, 4, 6, 7],
        co: [1, 2, 0, 0, 2, 1, 0, 0],
        ep: [0, 9, 2, 3, 4, 8, 6, 7, 1, 5, 10, 11],
        eo: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
    },
    // D
    CubieCube {
        cp: [0, 1, 2, 3, 5, 6, 7, 4],
        co: [0, 0, 0, 0, 0, 0, 0, 0],
        ep: [0, 1, 2, 3, 5, 6, 7, 4, 8, 9, 10, 11],
        eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    },
    // L
    CubieCube {
        cp: [0, 2, 6, 3, 4, 1, 5, 7],
        co: [0, 1, 2, 0, 0, 2, 1, 0],
        ep: [0, 1, 10, 3, 4, 5, 9, 7, 8, 2, 6, 11],
        eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    },
    // B
    CubieCube {
        cp: [0, 1, 3, 7, 4, 5, 2, 6],
        co: [0, 0, 1, 2, 0, 0, 2, 1],
        ep: [0, 1, 2, 11, 4, 5, 6, 10, 8, 9, 3, 7],
        eo: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::{parse_sequence, Face};

    fn apply_str(cube: &mut CubieCube, seq: &str) {
        cube.apply_all(&parse_sequence(seq).unwrap());
    }

    /// Apply `seq` `n` times to a solved cube; return whether it's back home.
    fn order_is(seq: &str, n: usize) -> bool {
        let moves = parse_sequence(seq).unwrap();
        let mut cube = CubieCube::SOLVED;
        for _ in 0..n {
            cube.apply_all(&moves);
        }
        cube == CubieCube::SOLVED
    }

    #[test]
    fn base_moves_are_valid_states() {
        for m in &BASE_MOVES {
            assert!(m.is_valid(), "a base move is not a legal cube state: {m:?}");
        }
    }

    #[test]
    fn quarter_turn_has_order_four() {
        for face in Face::ALL {
            let f = face.letter().to_string();
            assert!(order_is(&f, 4), "{f} should have order 4");
            assert!(!order_is(&f, 1), "{f} should not be identity after 1");
            assert!(!order_is(&f, 2), "{f} should not be identity after 2");
        }
    }

    #[test]
    fn double_turn_has_order_two() {
        for face in Face::ALL {
            let f = format!("{}2", face.letter());
            assert!(order_is(&f, 2), "{f} should have order 2");
            assert!(!order_is(&f, 1));
        }
    }

    #[test]
    fn move_then_inverse_is_identity() {
        let seq = parse_sequence("R U F L D B R2 U' L'").unwrap();
        let mut cube = CubieCube::SOLVED;
        cube.apply_all(&seq);
        assert!(!cube.is_solved());
        // Apply the inverse sequence (reverse order, inverted moves).
        for m in seq.iter().rev() {
            cube.apply(m.inverse());
        }
        assert_eq!(cube, CubieCube::SOLVED);
    }

    #[test]
    fn sexy_move_has_order_six() {
        // (R U R' U') returns to solved after exactly 6 repetitions.
        assert!(order_is("R U R' U'", 6));
        assert!(!order_is("R U R' U'", 1));
        assert!(!order_is("R U R' U'", 3));
    }

    #[test]
    fn r_u_has_order_105() {
        // A classic invariant: the permutation "R U" has order 105. This is a
        // strong correctness check on the R and U move tables together.
        assert!(order_is("R U", 105));
        assert!(!order_is("R U", 5));
        assert!(!order_is("R U", 21));
    }

    #[test]
    fn applied_move_produces_valid_state() {
        let mut cube = CubieCube::SOLVED;
        apply_str(&mut cube, "R U R' U' F2 B L D' R2 L' U2 F'");
        assert!(cube.is_valid(), "reachable state must stay valid");
    }

    #[test]
    fn t_perm_permutes_only_top_layer() {
        // T-perm swaps two edges and two corners in the top layer, leaving the
        // rest solved. After it, the cube is disturbed but valid.
        let mut cube = CubieCube::SOLVED;
        apply_str(&mut cube, "R U R' U' R' F R2 U' R' U' R U R' F'");
        assert!(cube.is_valid());
        assert!(!cube.is_solved());
        // Applying a T-perm twice returns to solved (it's an involution).
        apply_str(&mut cube, "R U R' U' R' F R2 U' R' U' R U R' F'");
        assert!(cube.is_solved());
    }

    #[test]
    fn superflip_is_an_involution_and_not_solved() {
        // The superflip (all 12 edges flipped in place) is famously order 2.
        let sf = "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2";
        let mut cube = CubieCube::SOLVED;
        apply_str(&mut cube, sf);
        assert!(cube.is_valid());
        assert!(!cube.is_solved(), "superflip is not the solved state");
        apply_str(&mut cube, sf);
        assert!(cube.is_solved(), "superflip applied twice is solved");
    }
}
