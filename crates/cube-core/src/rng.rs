//! A tiny, dependency-free, deterministic PRNG.
//!
//! Scrambles must be reproducible (same seed → same scramble, for shared
//! challenges and tests) and the crate must stay `no-dependency` and
//! wasm-friendly, so we use SplitMix64 rather than pulling in `rand`. The
//! browser supplies a high-entropy `u64` seed (e.g. from `crypto`); tests use
//! fixed seeds.

/// SplitMix64 generator. Fast, tiny, and good enough for scramble selection
/// (it is *not* cryptographically secure — it does not need to be).
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Create a generator from a 64-bit seed. Every seed is valid.
    pub fn new(seed: u64) -> SplitMix64 {
        SplitMix64 { state: seed }
    }

    /// Next 64-bit output.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniformly distributed integer in `0..n`, via rejection sampling so
    /// there is no modulo bias. Panics if `n == 0`.
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "below(0) is undefined");
        // `limit` is the largest multiple of `n` that fits in a u64; drawing
        // uniformly from `0..limit` and taking `% n` is unbiased.
        let limit = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < limit {
                return x % n;
            }
        }
    }

    /// Fisher–Yates shuffle: uniformly permute `slice` in place.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.below((i + 1) as u64) as usize;
            slice.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_a_seed() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = SplitMix64::new(1);
        let mut b = SplitMix64::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn below_stays_in_range() {
        let mut r = SplitMix64::new(7);
        for n in 1..=13u64 {
            for _ in 0..1000 {
                assert!(r.below(n) < n);
            }
        }
    }

    #[test]
    fn below_is_roughly_uniform() {
        // Deterministic (fixed seed), so this can never flake.
        let mut r = SplitMix64::new(12345);
        let mut counts = [0u32; 6];
        let n = 60_000;
        for _ in 0..n {
            counts[r.below(6) as usize] += 1;
        }
        let expected = n / 6; // 10_000
        for c in counts {
            assert!((c as i32 - expected as i32).abs() < 500, "bucket {c} too skewed");
        }
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut r = SplitMix64::new(99);
        let mut v: Vec<u8> = (0..12).collect();
        r.shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, (0..12).collect::<Vec<_>>());
        // With this seed the order should actually change (sanity, not a law).
        assert_ne!(v, (0..12).collect::<Vec<_>>());
    }
}
