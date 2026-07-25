// The Three.js view: 27 fixed cubies whose sticker colors are repainted from
// the wasm `facelets()` after every move (the wasm state is the source of
// truth). Moves are shown by rotating a temporary pivot group, then snapping the
// cubies home and repainting — visually seamless because a 90°/180° turn lands
// stickers exactly where the repaint puts them.

import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import {
  FACELET_MAP,
  FACE_MATERIAL,
  PLASTIC,
  STICKER_COLORS,
} from "./facelets";
import type { Face, Move } from "./moves";
import { FACE_ROT, type Axis } from "./rotations";

const SPACING = 1.05;
const CUBIE = 0.96;
const TURN_MS = 150;

export interface Hit {
  grid: [number, number, number];
  normal: THREE.Vector3;
  point: THREE.Vector3;
}

function comp(v: THREE.Vector3, a: Axis): number {
  return a === "x" ? v.x : a === "y" ? v.y : v.z;
}

function unit(a: Axis): THREE.Vector3 {
  return new THREE.Vector3(a === "x" ? 1 : 0, a === "y" ? 1 : 0, a === "z" ? 1 : 0);
}

export class CubeScene {
  private renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private camera: THREE.PerspectiveCamera;
  private controls: OrbitControls;
  private raycaster = new THREE.Raycaster();
  /** Cubie meshes keyed by "x,y,z" grid position. */
  private cubies = new Map<string, THREE.Mesh>();
  private animating = false;

  constructor(private canvas: HTMLCanvasElement) {
    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

    this.scene.background = new THREE.Color(0x0d0d10);
    this.camera = new THREE.PerspectiveCamera(45, 1, 0.1, 100);
    this.camera.position.set(4, 4.2, 5.2);

    this.scene.add(new THREE.HemisphereLight(0xffffff, 0x444455, 1.35));
    const dir = new THREE.DirectionalLight(0xffffff, 0.6);
    dir.position.set(5, 8, 6);
    this.scene.add(dir);

    this.controls = new OrbitControls(this.camera, canvas);
    this.controls.enablePan = false;
    this.controls.enableZoom = true;
    this.controls.minDistance = 4;
    this.controls.maxDistance = 12;
    this.controls.rotateSpeed = 0.9;

    this.buildCubies();
    this.resize();
    window.addEventListener("resize", () => this.resize());
    this.renderer.setAnimationLoop(() => {
      this.controls.update();
      this.renderer.render(this.scene, this.camera);
    });
  }

  private key(x: number, y: number, z: number): string {
    return `${x},${y},${z}`;
  }

  private buildCubies(): void {
    const geo = new THREE.BoxGeometry(CUBIE, CUBIE, CUBIE);
    for (let x = -1; x <= 1; x++) {
      for (let y = -1; y <= 1; y++) {
        for (let z = -1; z <= 1; z++) {
          const materials = Array.from(
            { length: 6 },
            () => new THREE.MeshStandardMaterial({ color: PLASTIC, roughness: 0.35, metalness: 0 }),
          );
          const mesh = new THREE.Mesh(geo, materials);
          mesh.position.set(x * SPACING, y * SPACING, z * SPACING);
          mesh.userData.grid = [x, y, z];
          this.cubies.set(this.key(x, y, z), mesh);
          this.scene.add(mesh);
        }
      }
    }
  }

  /** Repaint every sticker from a 54-entry facelet-color array. */
  repaint(facelets: Uint8Array): void {
    for (const mesh of this.cubies.values()) {
      for (const m of mesh.material as THREE.MeshStandardMaterial[]) {
        m.color.setHex(PLASTIC);
        m.emissive.setHex(0x000000);
      }
    }
    for (let idx = 0; idx < 54; idx++) {
      const s = FACELET_MAP[idx];
      const mesh = this.cubies.get(this.key(s.pos[0], s.pos[1], s.pos[2]))!;
      const mat = (mesh.material as THREE.MeshStandardMaterial[])[FACE_MATERIAL[s.face]];
      mat.color.setHex(STICKER_COLORS[facelets[idx]]);
    }
  }

  /** Rotate the layer for `move`, resolving when the tween finishes. */
  animateMove(move: Move): Promise<void> {
    const fr = FACE_ROT[move.face];
    const layer = [...this.cubies.values()].filter(
      (m) => (m.userData.grid as number[])[fr.axis === "x" ? 0 : fr.axis === "y" ? 1 : 2] === fr.layer,
    );
    const pivot = new THREE.Group();
    this.scene.add(pivot);
    for (const m of layer) pivot.add(m); // keeps local transforms; world unchanged

    const sign = move.turn === 3 ? -1 : 1;
    const mult = move.turn === 2 ? 2 : 1;
    const target = (Math.PI / 2) * fr.cw * sign * mult;
    const dur = TURN_MS * (move.turn === 2 ? 1.4 : 1);
    const axis = fr.axis;

    return new Promise<void>((resolve) => {
      this.animating = true;
      const start = performance.now();
      const tick = (now: number) => {
        const t = Math.min(1, (now - start) / dur);
        const e = t * t * (3 - 2 * t); // smoothstep
        const val = target * e;
        if (axis === "x") pivot.rotation.x = val;
        else if (axis === "y") pivot.rotation.y = val;
        else pivot.rotation.z = val;
        if (t < 1) {
          requestAnimationFrame(tick);
        } else {
          for (const m of layer) this.scene.add(m); // snap home
          this.scene.remove(pivot);
          this.animating = false;
          resolve();
        }
      };
      requestAnimationFrame(tick);
    });
  }

  /** Briefly glow every sticker of a face (used by Hint). */
  flashFace(face: Face): Promise<void> {
    const mats: THREE.MeshStandardMaterial[] = [];
    for (let idx = 0; idx < 54; idx++) {
      const s = FACELET_MAP[idx];
      if (s.face !== face) continue;
      const mesh = this.cubies.get(this.key(s.pos[0], s.pos[1], s.pos[2]))!;
      mats.push((mesh.material as THREE.MeshStandardMaterial[])[FACE_MATERIAL[s.face]]);
    }
    for (const m of mats) m.emissive.setHex(0x666666);
    return new Promise((resolve) =>
      setTimeout(() => {
        for (const m of mats) m.emissive.setHex(0x000000);
        resolve();
      }, 700),
    );
  }

  get isAnimating(): boolean {
    return this.animating;
  }

  setOrbitEnabled(on: boolean): void {
    this.controls.enabled = on;
  }

  private ndc(clientX: number, clientY: number): THREE.Vector2 {
    const r = this.canvas.getBoundingClientRect();
    return new THREE.Vector2(
      ((clientX - r.left) / r.width) * 2 - 1,
      -((clientY - r.top) / r.height) * 2 + 1,
    );
  }

  /** Raycast a pointer position against the cube; null if it misses. */
  raycast(clientX: number, clientY: number): Hit | null {
    this.raycaster.setFromCamera(this.ndc(clientX, clientY), this.camera);
    const hits = this.raycaster.intersectObjects([...this.cubies.values()], false);
    if (hits.length === 0) return null;
    const h = hits[0];
    const mesh = h.object as THREE.Mesh;
    const normal = h
      .face!.normal.clone()
      .transformDirection(mesh.matrixWorld)
      .round();
    return { grid: mesh.userData.grid as [number, number, number], normal, point: h.point.clone() };
  }

  /** World-space point where the pointer ray meets the plane of a hit face. */
  pointOnFacePlane(clientX: number, clientY: number, hit: Hit): THREE.Vector3 | null {
    const plane = new THREE.Plane().setFromNormalAndCoplanarPoint(hit.normal, hit.point);
    this.raycaster.setFromCamera(this.ndc(clientX, clientY), this.camera);
    const out = new THREE.Vector3();
    return this.raycaster.ray.intersectPlane(plane, out) ? out : null;
  }

  private resize(): void {
    const w = this.canvas.clientWidth || 1;
    const h = this.canvas.clientHeight || 1;
    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
  }
}

// Re-exported helpers for the drag-to-twist math in controls.ts.
export { comp, unit };
