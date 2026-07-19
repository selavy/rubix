//! # cube-core
//!
//! The Rust core for Rubix: a 3×3×3 Rubik's Cube state model and move engine,
//! plus (in later increments) scramble generation and Kociemba's two-phase
//! solver. Compiled to WebAssembly for the browser and usable as a plain Rust
//! library on the server. See `DESIGN.md` §4.
//!
//! ## What's implemented so far
//! - [`CubieCube`]: the corner/edge coordinate model and move composition.
//! - The 18 outer-face turns of the Half-Turn Metric (`U D L R F B` × cw/2/ccw).
//! - Standard [`moves`] notation parsing and formatting.
//! - The 54-sticker [`facelet`] view and the orientation-independent win check.
//! - Uniform random-state [`scramble`] generation (via the seedable [`rng`]).
//! - Kociemba's two-phase [`solver`] (`CubieCube::solve` / `hint`).
//!
//! ## Not yet implemented
//! - Wide turns, slice moves (`M E S`), and whole-cube rotations (`x y z`),
//!   which need a spatial-orientation tracker on top of the fixed-center model.
//! - The displayable scramble *move sequence* (solve a random state and invert
//!   it) — now unblocked by the solver.
//!
//! ## Example
//! ```
//! use cube_core::{CubieCube, moves::parse_sequence};
//!
//! let mut cube = CubieCube::SOLVED;
//! cube.apply_all(&parse_sequence("R U R' U'").unwrap());
//! assert!(!cube.is_solved());
//! ```

pub mod cubie;
pub mod facelet;
pub mod moves;
pub mod rng;
pub mod scramble;
pub mod solver;

pub use cubie::CubieCube;
pub use facelet::Color;
pub use moves::{Face, Move, Turn};
pub use rng::SplitMix64;
