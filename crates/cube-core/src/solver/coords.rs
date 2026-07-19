//! Coordinate encodings for Kociemba's two-phase algorithm.
//!
//! The solver never searches over raw cubes — it searches over small integer
//! *coordinates* that capture just the part of the state each phase cares about.
//! Each coordinate has a `get` (cube → integer) and a `set` (integer → a
//! representative cube), which together let us precompute move-transition and
//! pruning tables.
//!
//! Phase 1 drives the cube into the subgroup G1 = ⟨U, D, L2, R2, F2, B2⟩ using:
//! - **twist** — corner orientation, `0..2187` (3⁷).
//! - **flip** — edge orientation, `0..2048` (2¹¹).
//! - **slice** — which four positions hold the UD-slice edges, `0..495` (C(12,4)).
//!
//! Phase 2 solves within G1 using:
//! - **corners** — corner permutation, `0..40320` (8!).
//! - **edge8** — permutation of the eight U/D-layer edges, `0..40320` (8!).
//! - **sliceperm** — permutation of the four slice edges, `0..24` (4!).
//!
//! Every coordinate maps the solved cube to `0`, so all search goals are `0`.

use crate::cubie::CubieCube;

pub const N_TWIST: usize = 2187;
pub const N_FLIP: usize = 2048;
pub const N_SLICE: usize = 495;
pub const N_CORNER: usize = 40320;
pub const N_EDGE8: usize = 40320;
pub const N_SLICEPERM: usize = 24;

/// Binomial coefficient C(n, k), computed exactly, returning 0 when `k > n`.
pub fn cnk(n: u32, k: u32) -> u32 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut r: u64 = 1;
    for i in 0..k {
        // Each partial product is itself a binomial coefficient, so the
        // division is always exact.
        r = r * (n - i) as u64 / (i + 1) as u64;
    }
    r as u32
}

fn factorial(n: usize) -> u32 {
    const F: [u32; 9] = [1, 1, 2, 6, 24, 120, 720, 5040, 40320];
    F[n]
}

/// Lehmer-code rank of a permutation of `0..n` (identity ranks 0).
pub fn perm_rank(perm: &[u8]) -> u32 {
    let n = perm.len();
    let mut elems: Vec<u8> = (0..n as u8).collect();
    let mut rank = 0u32;
    for (i, &p) in perm.iter().enumerate() {
        let pos = elems.iter().position(|&x| x == p).unwrap();
        rank += pos as u32 * factorial(n - 1 - i);
        elems.remove(pos);
    }
    rank
}

/// Inverse of [`perm_rank`]: the `rank`-th permutation of `0..n`.
pub fn perm_unrank(mut rank: u32, n: usize) -> Vec<u8> {
    let mut elems: Vec<u8> = (0..n as u8).collect();
    let mut perm = Vec::with_capacity(n);
    for i in 0..n {
        let f = factorial(n - 1 - i);
        let pos = (rank / f) as usize;
        rank %= f;
        perm.push(elems.remove(pos));
    }
    perm
}

// --- twist (corner orientation) -----------------------------------------

pub fn get_twist(c: &CubieCube) -> u16 {
    let mut ret = 0u16;
    for i in 0..7 {
        ret = 3 * ret + c.co[i] as u16;
    }
    ret
}

pub fn set_twist(c: &mut CubieCube, mut twist: u16) {
    let mut parity = 0u16;
    for i in (0..7).rev() {
        let o = (twist % 3) as u8;
        c.co[i] = o;
        parity += o as u16;
        twist /= 3;
    }
    c.co[7] = ((3 - parity % 3) % 3) as u8;
}

// --- flip (edge orientation) --------------------------------------------

pub fn get_flip(c: &CubieCube) -> u16 {
    let mut ret = 0u16;
    for i in 0..11 {
        ret = 2 * ret + c.eo[i] as u16;
    }
    ret
}

pub fn set_flip(c: &mut CubieCube, mut flip: u16) {
    let mut parity = 0u16;
    for i in (0..11).rev() {
        let o = (flip % 2) as u8;
        c.eo[i] = o;
        parity += o as u16;
        flip /= 2;
    }
    c.eo[11] = (parity % 2) as u8;
}

// --- slice (UD-slice edge positions, order-independent) -----------------

pub fn get_slice(c: &CubieCube) -> u16 {
    // Combinatorial number system over the "distance from the back" of each
    // slice-edge position, so the solved cube (slice edges at 8..11) maps to 0.
    let mut idx = 0u32;
    let mut t = 0u32;
    for p in (0..12).rev() {
        if c.ep[p] >= 8 {
            idx += cnk((11 - p) as u32, t + 1);
            t += 1;
        }
    }
    idx as u16
}

pub fn set_slice(c: &mut CubieCube, idx: u16) {
    // Decode the combination, then place the four slice edges at those
    // positions and fill the rest with the U/D edges in order. Only the slice
    // *positions* matter for this coordinate, so any fixed order works.
    let mut rem = idx as u32;
    let mut q = [0usize; 4];
    for t in (0..4usize).rev() {
        let mut cc = t as u32;
        while cnk(cc + 1, t as u32 + 1) <= rem {
            cc += 1;
        }
        q[t] = cc as usize;
        rem -= cnk(cc, t as u32 + 1);
    }
    let mut ep = [255u8; 12];
    let slice_edge = [8u8, 9, 10, 11];
    for (t, &qt) in q.iter().enumerate() {
        ep[11 - qt] = slice_edge[t];
    }
    let other = [0u8, 1, 2, 3, 4, 5, 6, 7];
    let mut y = 0;
    for slot in ep.iter_mut() {
        if *slot == 255 {
            *slot = other[y];
            y += 1;
        }
    }
    c.ep = ep;
}

// --- corners (permutation) ----------------------------------------------

pub fn get_corners(c: &CubieCube) -> u16 {
    perm_rank(&c.cp) as u16
}

pub fn set_corners(c: &mut CubieCube, idx: u16) {
    let p = perm_unrank(idx as u32, 8);
    c.cp.copy_from_slice(&p);
}

// --- edge8 (U/D-layer edge permutation) ---------------------------------

pub fn get_edge8(c: &CubieCube) -> u16 {
    perm_rank(&c.ep[0..8]) as u16
}

pub fn set_edge8(c: &mut CubieCube, idx: u16) {
    let p = perm_unrank(idx as u32, 8);
    c.ep[0..8].copy_from_slice(&p);
    c.ep[8..12].copy_from_slice(&[8, 9, 10, 11]);
}

// --- sliceperm (slice edge permutation) ---------------------------------

pub fn get_sliceperm(c: &CubieCube) -> u16 {
    let arr = [c.ep[8] - 8, c.ep[9] - 8, c.ep[10] - 8, c.ep[11] - 8];
    perm_rank(&arr) as u16
}

pub fn set_sliceperm(c: &mut CubieCube, idx: u16) {
    let p = perm_unrank(idx as u32, 4);
    for i in 0..8 {
        c.ep[i] = i as u8;
    }
    for i in 0..4 {
        c.ep[8 + i] = p[i] + 8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cnk_known_values() {
        assert_eq!(cnk(12, 4), 495);
        assert_eq!(cnk(0, 0), 1);
        assert_eq!(cnk(5, 0), 1);
        assert_eq!(cnk(4, 5), 0);
        assert_eq!(cnk(11, 4), 330);
    }

    #[test]
    fn perm_rank_unrank_roundtrip_n8() {
        // Exhaustive over all 8! permutations: rank∘unrank and unrank∘rank.
        for idx in 0..40320u32 {
            let p = perm_unrank(idx, 8);
            assert_eq!(perm_rank(&p), idx);
        }
    }

    #[test]
    fn solved_maps_every_coordinate_to_zero() {
        let s = CubieCube::SOLVED;
        assert_eq!(get_twist(&s), 0);
        assert_eq!(get_flip(&s), 0);
        assert_eq!(get_slice(&s), 0);
        assert_eq!(get_corners(&s), 0);
        assert_eq!(get_edge8(&s), 0);
        assert_eq!(get_sliceperm(&s), 0);
    }

    #[test]
    fn twist_roundtrips() {
        for x in 0..N_TWIST as u16 {
            let mut c = CubieCube::SOLVED;
            set_twist(&mut c, x);
            assert_eq!(get_twist(&c), x);
            assert!(c.co.iter().map(|&o| o as u32).sum::<u32>() % 3 == 0);
        }
    }

    #[test]
    fn flip_roundtrips() {
        for x in 0..N_FLIP as u16 {
            let mut c = CubieCube::SOLVED;
            set_flip(&mut c, x);
            assert_eq!(get_flip(&c), x);
            assert!(c.eo.iter().map(|&o| o as u32).sum::<u32>() % 2 == 0);
        }
    }

    #[test]
    fn slice_roundtrips() {
        for x in 0..N_SLICE as u16 {
            let mut c = CubieCube::SOLVED;
            set_slice(&mut c, x);
            assert_eq!(get_slice(&c), x, "slice coord {x} did not round-trip");
        }
    }

    #[test]
    fn corner_and_sliceperm_roundtrip() {
        for x in [0u16, 1, 100, 40319] {
            let mut c = CubieCube::SOLVED;
            set_corners(&mut c, x);
            assert_eq!(get_corners(&c), x);
        }
        for x in 0..N_SLICEPERM as u16 {
            let mut c = CubieCube::SOLVED;
            set_sliceperm(&mut c, x);
            assert_eq!(get_sliceperm(&c), x);
        }
    }

    #[test]
    fn edge8_roundtrips_sampled() {
        for x in [0u16, 1, 2, 999, 20000, 40319] {
            let mut c = CubieCube::SOLVED;
            set_edge8(&mut c, x);
            assert_eq!(get_edge8(&c), x);
        }
    }
}
