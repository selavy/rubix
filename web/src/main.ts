// Entry point: initialize the wasm engine, build the scene/game/UI, and wire
// input to a serialized move queue so animations never overlap.

import init, { Cube } from "./pkg/cube_wasm.js";
import { CubeScene } from "./scene";
import { Game } from "./game";
import { UI } from "./ui";
import { initControls } from "./controls";
import { invertMove, type Move } from "./moves";
import "./styles.css";

await init();

const canvas = document.getElementById("scene") as HTMLCanvasElement;
const cube = new Cube();
const game = new Game(cube);
const scene = new CubeScene(canvas);

// All cube-mutating work runs on this promise chain, one step at a time.
let chain: Promise<void> = Promise.resolve();
const enqueue = (step: () => Promise<void>): Promise<void> => {
  chain = chain.then(step).catch((err) => console.error(err));
  return chain;
};

let ui: UI;

function afterChange(): void {
  scene.repaint(game.facelets());
  ui.refresh();
  if (game.isSolved() && game.count > 0) ui.showComplete();
}

// A single user/solution move: animate, then commit to the engine.
function performMove(m: Move): Promise<void> {
  return enqueue(async () => {
    await scene.animateMove(m);
    game.commit(m);
    afterChange();
  });
}

function undo(): void {
  enqueue(async () => {
    const m = game.popUndo();
    if (!m) return;
    const inv = invertMove(m);
    await scene.animateMove(inv);
    game.applyRaw(inv);
    ui.hideComplete();
    afterChange();
  });
}

function redo(): void {
  enqueue(async () => {
    const m = game.popRedo();
    if (!m) return;
    await scene.animateMove(m);
    game.applyRaw(m);
    afterChange();
  });
}

function newScramble(): void {
  enqueue(async () => {
    game.scramble();
    ui.hideComplete();
    ui.clearSolution();
    ui.clearHint();
    afterChange();
  });
}

function resetToScramble(): void {
  enqueue(async () => {
    game.resetToScramble();
    ui.hideComplete();
    ui.clearSolution();
    ui.clearHint();
    afterChange();
  });
}

function hint(): void {
  const m = game.hintMove();
  if (!m) return;
  ui.showHint(m);
  enqueue(() => scene.flashFace(m.face));
}

function solve(): void {
  const seq = game.solveSequence();
  if (seq.length === 0) return;
  ui.showSolution(seq);
  for (const m of seq) performMove(m);
}

ui = new UI(game, {
  scramble: newScramble,
  reset: resetToScramble,
  undo,
  redo,
  hint,
  solve,
});

initControls(scene, canvas, performMove);

// Start with a fresh scramble.
game.scramble();
afterChange();
