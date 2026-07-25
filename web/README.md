# Rubix — Frontend

TypeScript + Three.js app that renders the cube and drives the `cube-core`
WebAssembly engine. See `../DESIGN.md`.

## Prerequisites

- Rust toolchain with the `wasm32-unknown-unknown` target
  (`rustup target add wasm32-unknown-unknown`)
- `wasm-bindgen-cli` matching the `wasm-bindgen` crate version
  (`cargo install wasm-bindgen-cli --version <ver>`)
- Node 18+

## Develop

```sh
npm install
npm run build:wasm   # compile the Rust core to wasm + generate src/pkg (rerun when cube-core changes)
npm run dev          # start the Vite dev server
```

Then open the printed URL.

## Other commands

- `npm run typecheck` — type-check without emitting.
- `npm run build` — type-check and produce a production bundle in `dist/`.

## Controls

- **Drag a face** to twist that layer; **drag the background** to rotate the view.
- **Keyboard:** a face letter (`U D L R F B`) turns it clockwise; hold `Shift`
  for counter-clockwise. `Backspace` = undo, `Esc` closes overlays.
- **Buttons:** New Scramble, Reset (back to the scramble), Undo/Redo, Hint, Solve.

## Layout

| File            | Responsibility                                             |
| --------------- | ---------------------------------------------------------- |
| `facelets.ts`   | Exact facelet-index → 3D sticker map (from the Rust tables) |
| `rotations.ts`  | Face-turn ↔ world-axis rotation geometry                   |
| `scene.ts`      | Three.js scene, repaint-from-facelets, move animation, raycast |
| `game.ts`       | Wasm `Cube` wrapper: history, undo/redo, scramble, timer   |
| `controls.ts`   | Keyboard + pointer drag-to-twist                           |
| `ui.ts`         | Move panel, counter, timer, buttons, overlays              |
| `main.ts`       | Wires it together with a serialized move queue             |
