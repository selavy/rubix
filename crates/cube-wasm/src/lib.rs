//! WebAssembly bindings for the Rubix cube engine (`DESIGN.md` §3.2).
//!
//! This is a thin wrapper over [`cube_core`]: it owns a single [`CubieCube`] and
//! exposes exactly the operations the TypeScript/Three.js frontend needs. All
//! cube logic lives in `cube-core` (and is tested there); this layer only
//! translates types across the JS boundary.
//!
//! The frontend holds one [`Cube`] as the source of truth and renders whatever
//! [`Cube::facelets`] reports after each call.

use cube_core::moves::{format_sequence, parse_sequence};
use cube_core::CubieCube;
use wasm_bindgen::prelude::*;

/// A Rubik's Cube the browser can drive: apply moves, scramble, query stickers,
/// check for a win, and ask for a solution or a hint.
#[wasm_bindgen]
pub struct Cube {
    inner: CubieCube,
}

impl Default for Cube {
    fn default() -> Self {
        Cube { inner: CubieCube::SOLVED }
    }
}

#[wasm_bindgen]
impl Cube {
    /// Create a solved cube: `new Cube()` in JS.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Cube {
        Cube::default()
    }

    /// Apply one or more whitespace-separated moves in standard notation
    /// (e.g. `"R"`, `"U'"`, `"R U R' U'"`). Rejects any unparseable token,
    /// leaving the cube unchanged. Only the outer face turns `U D L R F B`
    /// (with `'`/`2`) are supported today.
    pub fn apply(&mut self, moves: &str) -> Result<(), JsValue> {
        let seq = parse_sequence(moves).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner.apply_all(&seq);
        Ok(())
    }

    /// Scramble to a uniformly random state and return the scramble as notation.
    ///
    /// Pass a `seed` (a BigInt) for a reproducible scramble, or omit it for a
    /// fresh random one. The cube is left in the scrambled state; the returned
    /// string is the sequence that reaches it from solved.
    pub fn scramble(&mut self, seed: Option<u64>) -> String {
        let seed = seed.unwrap_or_else(random_seed);
        let (state, sequence) = CubieCube::scramble_with_sequence(seed);
        self.inner = state;
        format_sequence(&sequence)
    }

    /// The 54 sticker colors (values `0..6`) in facelet order `U R F D L B`,
    /// nine per face, row-major. The renderer maps each value to a color.
    pub fn facelets(&self) -> Vec<u8> {
        self.inner.facelets().iter().map(|&c| c as u8).collect()
    }

    /// Whether every face is a single color — the win condition.
    pub fn is_solved(&self) -> bool {
        self.inner.is_solved()
    }

    /// A short solution (Kociemba two-phase) as a notation string, or `""` if
    /// the cube is already solved. Errors only on an illegal cube.
    pub fn solve(&self) -> Result<String, JsValue> {
        match self.inner.solve() {
            Some(moves) => Ok(format_sequence(&moves)),
            None => Err(JsValue::from_str("cube is unsolvable")),
        }
    }

    /// Just the next move of the solution (for the Hint button), or `""` if the
    /// cube is already solved.
    pub fn hint(&self) -> Result<String, JsValue> {
        match self.inner.solve() {
            Some(moves) => Ok(moves.first().map(|m| m.to_string()).unwrap_or_default()),
            None => Err(JsValue::from_str("cube is unsolvable")),
        }
    }

    /// Reset to the solved state.
    pub fn reset(&mut self) {
        self.inner = CubieCube::SOLVED;
    }
}

/// A high-entropy `u64` seed from the JS PRNG, used when the caller does not
/// supply one. Not cryptographic — it only selects a scramble.
fn random_seed() -> u64 {
    let hi = (js_sys::Math::random() * (u32::MAX as f64)) as u64;
    let lo = (js_sys::Math::random() * (u32::MAX as f64)) as u64;
    (hi << 32) | lo
}
