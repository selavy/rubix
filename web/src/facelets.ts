// Maps each of the 54 wasm facelet indices to a physical sticker: which cubie
// (grid position in {-1,0,1}³) and which face it sits on.
//
// These entries are transcribed directly from cube-core's verified
// CORNER_FACELET / EDGE_FACELET / center tables (see facelet.rs), so the render
// is guaranteed to match the engine's notion of the cube — no orientation
// guesswork. Each corner contributes 3 stickers, each edge 2, each center 1.

import type { Face } from "./moves";

export interface Sticker {
  idx: number;
  face: Face;
  pos: [number, number, number];
}

// prettier-ignore
const STICKERS: Sticker[] = [
  // --- corners (URF UFL ULB UBR DFR DLF DBL DRB) ---
  { idx: 8, face: "U", pos: [1, 1, 1] }, { idx: 9, face: "R", pos: [1, 1, 1] }, { idx: 20, face: "F", pos: [1, 1, 1] },
  { idx: 6, face: "U", pos: [-1, 1, 1] }, { idx: 18, face: "F", pos: [-1, 1, 1] }, { idx: 38, face: "L", pos: [-1, 1, 1] },
  { idx: 0, face: "U", pos: [-1, 1, -1] }, { idx: 36, face: "L", pos: [-1, 1, -1] }, { idx: 47, face: "B", pos: [-1, 1, -1] },
  { idx: 2, face: "U", pos: [1, 1, -1] }, { idx: 45, face: "B", pos: [1, 1, -1] }, { idx: 11, face: "R", pos: [1, 1, -1] },
  { idx: 29, face: "D", pos: [1, -1, 1] }, { idx: 26, face: "F", pos: [1, -1, 1] }, { idx: 15, face: "R", pos: [1, -1, 1] },
  { idx: 27, face: "D", pos: [-1, -1, 1] }, { idx: 44, face: "L", pos: [-1, -1, 1] }, { idx: 24, face: "F", pos: [-1, -1, 1] },
  { idx: 33, face: "D", pos: [-1, -1, -1] }, { idx: 53, face: "B", pos: [-1, -1, -1] }, { idx: 42, face: "L", pos: [-1, -1, -1] },
  { idx: 35, face: "D", pos: [1, -1, -1] }, { idx: 17, face: "R", pos: [1, -1, -1] }, { idx: 51, face: "B", pos: [1, -1, -1] },
  // --- edges (UR UF UL UB DR DF DL DB FR FL BL BR) ---
  { idx: 5, face: "U", pos: [1, 1, 0] }, { idx: 10, face: "R", pos: [1, 1, 0] },
  { idx: 7, face: "U", pos: [0, 1, 1] }, { idx: 19, face: "F", pos: [0, 1, 1] },
  { idx: 3, face: "U", pos: [-1, 1, 0] }, { idx: 37, face: "L", pos: [-1, 1, 0] },
  { idx: 1, face: "U", pos: [0, 1, -1] }, { idx: 46, face: "B", pos: [0, 1, -1] },
  { idx: 32, face: "D", pos: [1, -1, 0] }, { idx: 16, face: "R", pos: [1, -1, 0] },
  { idx: 28, face: "D", pos: [0, -1, 1] }, { idx: 25, face: "F", pos: [0, -1, 1] },
  { idx: 30, face: "D", pos: [-1, -1, 0] }, { idx: 43, face: "L", pos: [-1, -1, 0] },
  { idx: 34, face: "D", pos: [0, -1, -1] }, { idx: 52, face: "B", pos: [0, -1, -1] },
  { idx: 23, face: "F", pos: [1, 0, 1] }, { idx: 12, face: "R", pos: [1, 0, 1] },
  { idx: 21, face: "F", pos: [-1, 0, 1] }, { idx: 41, face: "L", pos: [-1, 0, 1] },
  { idx: 50, face: "B", pos: [-1, 0, -1] }, { idx: 39, face: "L", pos: [-1, 0, -1] },
  { idx: 48, face: "B", pos: [1, 0, -1] }, { idx: 14, face: "R", pos: [1, 0, -1] },
  // --- centers (U R F D L B) ---
  { idx: 4, face: "U", pos: [0, 1, 0] }, { idx: 13, face: "R", pos: [1, 0, 0] }, { idx: 22, face: "F", pos: [0, 0, 1] },
  { idx: 31, face: "D", pos: [0, -1, 0] }, { idx: 40, face: "L", pos: [-1, 0, 0] }, { idx: 49, face: "B", pos: [0, 0, -1] },
];

/** Facelet index → sticker placement, ordered by index for O(1) lookup. */
export const FACELET_MAP: Sticker[] = (() => {
  const map = new Array<Sticker>(54);
  for (const s of STICKERS) map[s.idx] = s;
  return map;
})();

// BoxGeometry material order is [+x, -x, +y, -y, +z, -z] = [R, L, U, D, F, B].
export const FACE_MATERIAL: Record<Face, number> = { R: 0, L: 1, U: 2, D: 3, F: 4, B: 5 };

// Facelet color *values* (0..5) are the home face of each sticker, in the order
// U R F D L B. Mapped here to the standard Western scheme.
export const STICKER_COLORS: number[] = [
  0xf5f5f5, // U white
  0xc41e3a, // R red
  0x009e60, // F green
  0xffd500, // D yellow
  0xff5800, // L orange
  0x0051ba, // B blue
];

/** Interior (non-sticker) plastic color. */
export const PLASTIC = 0x141414;
