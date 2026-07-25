// Node smoke test for the wasm boundary. Exercises the Cube API end-to-end
// against the real compiled WebAssembly module.
//
//   1. Build:    cargo build -p cube-wasm --target wasm32-unknown-unknown --release
//   2. Bindgen:  wasm-bindgen target/wasm32-unknown-unknown/release/cube_wasm.wasm \
//                  --out-dir crates/cube-wasm/pkg --target nodejs
//   3. Run:      node crates/cube-wasm/smoke_test.cjs
//
// See crates/cube-wasm/build.sh, which runs all three.

const assert = require('node:assert');
const { Cube } = require('./pkg/cube_wasm.js');

// 1. A fresh cube is solved.
const c = new Cube();
assert.strictEqual(c.is_solved(), true, 'new cube should be solved');

// 2. facelets(): 54 stickers, each face uniform; solved face k has color k.
const f = c.facelets();
assert.strictEqual(f.length, 54, 'expected 54 facelets');
for (let face = 0; face < 6; face++) {
  for (let i = 0; i < 9; i++) {
    assert.strictEqual(f[face * 9 + i], face, `face ${face} sticker ${i}`);
  }
}

// 3. A move breaks the solved state; its inverse restores it.
c.apply('R');
assert.strictEqual(c.is_solved(), false, "R should un-solve");
c.apply("R'");
assert.strictEqual(c.is_solved(), true, "R then R' should be solved");

// 4. solve() output actually solves an applied scramble.
c.apply("R U R' U' F2 B L D' R2 U2 F'");
assert.strictEqual(c.is_solved(), false);
const solution = c.solve();
assert.ok(solution.length > 0, 'expected a non-empty solution');
c.apply(solution);
assert.strictEqual(c.is_solved(), true, 'applying solve() should solve the cube');

// 5. A seeded scramble is reproducible, and the returned notation reproduces
//    the same state when replayed on a fresh cube.
const a = new Cube();
const scrambleA = a.scramble(42n);
const b = new Cube();
const scrambleB = b.scramble(42n);
assert.strictEqual(scrambleA, scrambleB, 'same seed -> same scramble');
assert.strictEqual(a.is_solved(), false, 'scramble should un-solve');

const replay = new Cube();
replay.apply(scrambleA);
assert.deepStrictEqual(
  Array.from(replay.facelets()),
  Array.from(a.facelets()),
  'replaying the scramble notation should reproduce the state'
);

// 6. hint() is the first move of solve().
const h = new Cube();
h.scramble(7n);
assert.strictEqual(h.hint(), h.solve().split(' ')[0], 'hint should be first solve move');

// 7. A solved cube has an empty solution and hint.
const s = new Cube();
assert.strictEqual(s.solve(), '', 'solved cube solves in zero moves');
assert.strictEqual(s.hint(), '', 'solved cube has no hint');

// 8. An unparseable move is rejected (and leaves the cube unchanged).
assert.throws(() => new Cube().apply('Z'), 'invalid move should throw');

console.log('All wasm smoke tests passed.');
