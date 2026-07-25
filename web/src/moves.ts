// Move notation, mirroring cube-core's `moves` module on the TypeScript side.

export type Face = "U" | "R" | "F" | "D" | "L" | "B";
/** 1 = clockwise, 2 = 180°, 3 = counter-clockwise. */
export type Turn = 1 | 2 | 3;

export interface Move {
  face: Face;
  turn: Turn;
}

export function moveToString(m: Move): string {
  return m.face + (m.turn === 2 ? "2" : m.turn === 3 ? "'" : "");
}

export function parseMove(s: string): Move {
  const face = s[0] as Face;
  const suffix = s.slice(1);
  const turn: Turn = suffix === "2" ? 2 : suffix === "'" ? 3 : 1;
  return { face, turn };
}

export function invertMove(m: Move): Move {
  return { face: m.face, turn: m.turn === 1 ? 3 : m.turn === 3 ? 1 : 2 };
}

export function parseSequence(s: string): Move[] {
  return s
    .split(/\s+/)
    .filter(Boolean)
    .map(parseMove);
}

const FACES = "URFDLB";

/** Whether `k` (a keyboard key) names a face. */
export function isFaceKey(k: string): boolean {
  return k.length === 1 && FACES.includes(k.toUpperCase());
}
