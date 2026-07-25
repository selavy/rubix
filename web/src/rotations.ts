// The geometry that ties face turns to world-space rotations. Shared by the
// animation (scene) and the drag-to-twist input (controls).
//
// World axes: +x = R face, +y = U face, +z = F face. A clockwise face turn
// (viewed from outside the cube) is a negative rotation about that face's
// outward axis, so e.g. U = −90° about +y. Derived once here, used everywhere.

import type { Face } from "./moves";

export type Axis = "x" | "y" | "z";

export interface FaceRot {
  /** World axis the layer rotates about. */
  axis: Axis;
  /** Which slice (grid coordinate along `axis`): +1 or −1. */
  layer: number;
  /** Sign of a *clockwise* turn about the positive axis (−1 or +1). */
  cw: number;
}

export const FACE_ROT: Record<Face, FaceRot> = {
  U: { axis: "y", layer: 1, cw: -1 },
  D: { axis: "y", layer: -1, cw: 1 },
  R: { axis: "x", layer: 1, cw: -1 },
  L: { axis: "x", layer: -1, cw: 1 },
  F: { axis: "z", layer: 1, cw: -1 },
  B: { axis: "z", layer: -1, cw: 1 },
};

/** The face on the positive / negative side of each axis. */
export const FACE_OF: Record<Axis, { pos: Face; neg: Face }> = {
  x: { pos: "R", neg: "L" },
  y: { pos: "U", neg: "D" },
  z: { pos: "F", neg: "B" },
};
