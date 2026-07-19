//! Kociemba's two-phase solver (`DESIGN.md` §5).
//!
//! [`CubieCube::solve`](crate::CubieCube::solve) returns a short solution — the
//! first one the two-phase search finds. It is fast (well under a second once
//! tables are built) and typically ~21–24 moves in the Half-Turn Metric; it is
//! *not* guaranteed minimal. The large lookup tables are built once, lazily, and
//! shared across all solves.
//!
//! ```
//! use cube_core::CubieCube;
//! let scramble = CubieCube::scramble_from_seed(42);
//! let solution = scramble.solve().unwrap();
//! // Applying the solution actually solves the cube.
//! let mut c = scramble;
//! c.apply_all(&solution);
//! assert!(c.is_solved());
//! ```

mod coords;
mod search;
mod tables;

use std::sync::OnceLock;

use crate::cubie::CubieCube;
use crate::moves::Move;
use tables::Tables;

/// The lazily-built, process-wide solver tables (~4 MB).
static TABLES: OnceLock<Tables> = OnceLock::new();

fn tables() -> &'static Tables {
    TABLES.get_or_init(Tables::build)
}

impl CubieCube {
    /// A short solution for this cube via Kociemba's two-phase algorithm, as a
    /// sequence of face turns. Returns `Some([])` for an already-solved cube.
    ///
    /// The first call builds the solver tables (a one-time cost); later calls
    /// reuse them.
    pub fn solve(&self) -> Option<Vec<Move>> {
        search::solve(tables(), self)
    }

    /// The next single move of [`solve`](Self::solve), for the Hint button.
    pub fn hint(&self) -> Option<Move> {
        self.solve().and_then(|s| s.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use crate::cubie::CubieCube;
    use crate::moves::{format_sequence, parse_sequence};

    #[test]
    fn solved_cube_needs_no_moves() {
        assert_eq!(CubieCube::SOLVED.solve(), Some(vec![]));
        assert_eq!(CubieCube::SOLVED.hint(), None);
    }

    #[test]
    fn single_move_scramble_is_solved() {
        let mut c = CubieCube::SOLVED;
        c.apply_all(&parse_sequence("R").unwrap());
        let sol = c.solve().unwrap();
        let mut check = c;
        check.apply_all(&sol);
        assert!(check.is_solved());
        assert!(sol.len() <= 2, "a single turn should solve in ~1 move: {}", format_sequence(&sol));
    }

    #[test]
    fn known_scramble_is_solved() {
        let scramble = parse_sequence("R U R' U' F2 B L D' R2 L' U2 F'").unwrap();
        let mut c = CubieCube::SOLVED;
        c.apply_all(&scramble);
        let sol = c.solve().unwrap();
        let mut check = c;
        check.apply_all(&sol);
        assert!(check.is_solved(), "solution did not solve the cube");
    }

    #[test]
    fn superflip_is_solved() {
        let sf = "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2";
        let mut c = CubieCube::SOLVED;
        c.apply_all(&parse_sequence(sf).unwrap());
        let sol = c.solve().unwrap();
        let mut check = c;
        check.apply_all(&sol);
        assert!(check.is_solved());
    }

    #[test]
    fn random_scrambles_are_solved_within_bound() {
        // The definitive end-to-end check: every returned solution must
        // actually solve its scramble, and stay within a sane length.
        for seed in 0..80u64 {
            let scramble = CubieCube::scramble_from_seed(seed);
            let sol = scramble.solve().expect("every legal cube is solvable");
            let mut check = scramble;
            check.apply_all(&sol);
            assert!(check.is_solved(), "seed {seed}: solution does not solve the cube");
            assert!(
                sol.len() <= 30,
                "seed {seed}: solution unexpectedly long ({} moves)",
                sol.len()
            );
        }
    }

    #[test]
    fn hint_is_first_move_of_solution() {
        let scramble = CubieCube::scramble_from_seed(7);
        let sol = scramble.solve().unwrap();
        assert_eq!(scramble.hint(), sol.into_iter().next());
    }
}
