//! Random-state scramble generation (`DESIGN.md` §4.4).
//!
//! A WCA-style scramble is a *uniformly random legal cube state* — not the
//! result of applying N random moves, which is biased toward states reachable
//! in few moves. We sample the state directly in the coordinate model:
//!
//! 1. Uniform random corner and edge permutations (independent Fisher–Yates).
//! 2. Uniform random orientations, with the last corner twist and last edge
//!    flip forced so the twist sum is ≡ 0 (mod 3) and the flip sum ≡ 0 (mod 2).
//! 3. A single edge swap if needed so corner and edge permutation parities
//!    match. That conditional swap is a bijection on each parity class, so the
//!    result stays uniform over the 8!·12!·3⁷·2¹¹ ≈ 4.3×10¹⁹ legal states.
//!
//! For a virtual cube we set this state directly; the displayable scramble
//! *move sequence* (solve the state, invert it) will be produced once the
//! two-phase solver lands.

use crate::cubie::{permutation_parity, CubieCube};
use crate::rng::SplitMix64;

impl CubieCube {
    /// Sample a uniformly random *legal* cube state from `rng`.
    ///
    /// The result is guaranteed valid (passes [`CubieCube::is_valid`]) and,
    /// with overwhelming probability, not solved.
    pub fn random(rng: &mut SplitMix64) -> CubieCube {
        let mut cube = CubieCube::SOLVED;

        // 1. Independent uniform permutations of corners and edges.
        rng.shuffle(&mut cube.cp);
        rng.shuffle(&mut cube.ep);

        // 2a. Corner orientations: 7 free twists, the 8th forced so Σ ≡ 0 (3).
        let mut co_sum = 0u32;
        for slot in cube.co.iter_mut().take(7) {
            let twist = rng.below(3) as u8;
            *slot = twist;
            co_sum += twist as u32;
        }
        cube.co[7] = ((3 - (co_sum % 3)) % 3) as u8;

        // 2b. Edge orientations: 11 free flips, the 12th forced so Σ ≡ 0 (2).
        let mut eo_sum = 0u32;
        for slot in cube.eo.iter_mut().take(11) {
            let flip = rng.below(2) as u8;
            *slot = flip;
            eo_sum += flip as u32;
        }
        cube.eo[11] = (eo_sum % 2) as u8;

        // 3. Match permutation parity by swapping two edges if they disagree.
        //    Swapping two pieces flips only the edge parity, making the two
        //    equal — the legality condition for a solvable cube.
        if permutation_parity(&cube.cp) != permutation_parity(&cube.ep) {
            cube.ep.swap(0, 1);
        }

        debug_assert!(cube.is_valid(), "random() produced an illegal state");
        cube
    }

    /// Convenience: a random state from a raw `u64` seed. The browser passes a
    /// high-entropy seed; the same seed always yields the same scramble.
    pub fn scramble_from_seed(seed: u64) -> CubieCube {
        CubieCube::random(&mut SplitMix64::new(seed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_states_are_always_legal() {
        // Sweep many seeds; every sampled state must satisfy the cube invariants.
        for seed in 0..5_000u64 {
            let cube = CubieCube::scramble_from_seed(seed);
            assert!(cube.is_valid(), "seed {seed} produced an illegal state");
        }
    }

    #[test]
    fn reproducible_from_seed() {
        for seed in [0u64, 1, 7, 42, 1_000_000] {
            assert_eq!(
                CubieCube::scramble_from_seed(seed),
                CubieCube::scramble_from_seed(seed),
                "seed {seed} not reproducible"
            );
        }
    }

    #[test]
    fn distinct_seeds_give_distinct_states() {
        // Not a mathematical guarantee, but collisions are astronomically
        // unlikely; over a small sweep there should be none.
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for seed in 0..2_000u64 {
            let c = CubieCube::scramble_from_seed(seed);
            // Compare the full state (~4.3e19 possibilities), not just the
            // 8-corner permutation, which would collide by birthday paradox.
            assert!(seen.insert((c.cp, c.co, c.ep, c.eo)), "state collision at seed {seed}");
        }
    }

    #[test]
    fn scrambles_are_not_solved() {
        // A random legal state is solved with probability ~1/4.3e19.
        for seed in 0..5_000u64 {
            assert!(!CubieCube::scramble_from_seed(seed).is_solved());
        }
    }

    #[test]
    fn both_parities_occur() {
        // The parity fix must not pin every scramble to one parity class; the
        // corner-permutation parity should vary across seeds.
        let mut even = 0;
        let mut odd = 0;
        for seed in 0..2_000u64 {
            let c = CubieCube::scramble_from_seed(seed);
            match permutation_parity(&c.cp) {
                0 => even += 1,
                _ => odd += 1,
            }
        }
        assert!(even > 700 && odd > 700, "parity skew: even={even} odd={odd}");
    }

    #[test]
    fn corner_positions_are_roughly_uniform() {
        // Over many samples from one stream, each corner cubie should land in
        // slot 0 about 1/8 of the time. Deterministic stream ⇒ no flakiness.
        let mut rng = SplitMix64::new(2024);
        let mut counts = [0u32; 8];
        let n = 40_000;
        for _ in 0..n {
            let c = CubieCube::random(&mut rng);
            counts[c.cp[0] as usize] += 1;
        }
        let expected = (n / 8) as i32; // 5000
        for (cubie, &count) in counts.iter().enumerate() {
            assert!(
                (count as i32 - expected).abs() < 600,
                "corner {cubie} in slot 0: {count} (expected ~{expected})"
            );
        }
    }
}
