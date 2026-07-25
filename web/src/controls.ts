// Input: keyboard face turns, and pointer drag-to-twist. Dragging a face twists
// its layer; dragging the background orbits the camera (handled by
// OrbitControls, which we disable for the duration of a face grab).

import * as THREE from "three";
import { CubeScene, comp, unit, type Hit } from "./scene";
import { FACE_OF, FACE_ROT, type Axis } from "./rotations";
import { isFaceKey, type Face, type Move } from "./moves";

const DRAG_THRESHOLD = 0.22; // in cubie-spacing units

/** Infer the intended move from a face grab and an in-plane drag vector. */
function moveFromDrag(hit: Hit, drag: THREE.Vector3): Move | null {
  const n = hit.normal;
  // The two axes tangent to the grabbed face (n points along the third).
  const tangents = (["x", "y", "z"] as Axis[]).filter((a) => Math.abs(comp(n, a)) < 0.5);
  // Rotate about the tangent the drag is most perpendicular to (slides along
  // the other tangent).
  const [a0, a1] = tangents;
  const rotAxis: Axis = Math.abs(comp(drag, a0)) < Math.abs(comp(drag, a1)) ? a0 : a1;

  const layer = comp(new THREE.Vector3(...hit.grid), rotAxis);
  if (layer === 0) return null; // middle slice — not a face turn (unsupported)

  const face: Face = layer > 0 ? FACE_OF[rotAxis].pos : FACE_OF[rotAxis].neg;
  // Sign of the rotation about the positive axis that follows the drag.
  const tangentVel = unit(rotAxis).cross(n); // direction the surface moves for +rotation
  let s = Math.sign(drag.dot(tangentVel));
  if (s === 0) s = 1;
  const clockwise = s === FACE_ROT[face].cw;
  return { face, turn: clockwise ? 1 : 3 };
}

interface DragState {
  hit: Hit;
  pointerId: number;
  committed: boolean;
}

export function initControls(
  scene: CubeScene,
  canvas: HTMLCanvasElement,
  onMove: (m: Move) => void,
): void {
  // --- keyboard -----------------------------------------------------------
  window.addEventListener("keydown", (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const target = e.target as HTMLElement | null;
    if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA")) return;
    if (isFaceKey(e.key)) {
      onMove({ face: e.key.toUpperCase() as Face, turn: e.shiftKey ? 3 : 1 });
      e.preventDefault();
    }
  });

  // --- pointer drag-to-twist ---------------------------------------------
  let drag: DragState | null = null;

  // Capture phase so we can claim the gesture before OrbitControls sees it.
  canvas.addEventListener(
    "pointerdown",
    (ev) => {
      if (scene.isAnimating) return;
      const hit = scene.raycast(ev.clientX, ev.clientY);
      if (!hit) return; // background → let OrbitControls orbit
      drag = { hit, pointerId: ev.pointerId, committed: false };
      scene.setOrbitEnabled(false);
      canvas.setPointerCapture(ev.pointerId);
    },
    true,
  );

  canvas.addEventListener("pointermove", (ev) => {
    if (!drag || drag.committed) return;
    const p = scene.pointOnFacePlane(ev.clientX, ev.clientY, drag.hit);
    if (!p) return;
    const d = p.sub(drag.hit.point);
    if (d.length() < DRAG_THRESHOLD) return;
    const move = moveFromDrag(drag.hit, d);
    drag.committed = true;
    if (move) onMove(move);
  });

  const end = (ev: PointerEvent) => {
    if (!drag) return;
    scene.setOrbitEnabled(true);
    try {
      canvas.releasePointerCapture(ev.pointerId);
    } catch {
      /* pointer already released */
    }
    drag = null;
  };
  canvas.addEventListener("pointerup", end);
  canvas.addEventListener("pointercancel", end);
}
