// Game state: wraps the wasm Cube and tracks the user's move history, redo
// stack, scramble, and timer. Pure logic — no rendering.

import type { Cube } from "./pkg/cube_wasm.js";
import { moveToString, parseSequence, type Move } from "./moves";

export class Game {
  history: Move[] = [];
  private redoStack: Move[] = [];
  scrambleStr = "";
  private startTime: number | null = null;
  private solvedTime: number | null = null;

  constructor(private cube: Cube) {}

  facelets(): Uint8Array {
    return this.cube.facelets();
  }

  isSolved(): boolean {
    return this.cube.is_solved();
  }

  get count(): number {
    return this.history.length;
  }

  get canUndo(): boolean {
    return this.history.length > 0;
  }

  get canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  /** Apply a user move (already animated), recording it and starting the timer. */
  commit(m: Move): void {
    this.cube.apply(moveToString(m));
    this.history.push(m);
    this.redoStack = [];
    if (this.startTime === null) this.startTime = performance.now();
    this.checkSolved();
  }

  /** Apply a move to the wasm cube without touching the history (undo/redo). */
  applyRaw(m: Move): void {
    this.cube.apply(moveToString(m));
    this.checkSolved();
  }

  /** Pop the last move for undo; the caller animates and applies its inverse. */
  popUndo(): Move | null {
    const m = this.history.pop();
    if (!m) return null;
    this.redoStack.push(m);
    this.solvedTime = null; // undoing a solved cube un-solves it
    return m;
  }

  /** Pop a move for redo; the caller animates and applies it. */
  popRedo(): Move | null {
    const m = this.redoStack.pop();
    if (!m) return null;
    this.history.push(m);
    return m;
  }

  scramble(seed?: bigint): void {
    this.scrambleStr = seed === undefined ? this.cube.scramble() : this.cube.scramble(seed);
    this.resetProgress();
  }

  /** Return to the current scramble's starting position, clearing moves. */
  resetToScramble(): void {
    this.cube.reset();
    if (this.scrambleStr) this.cube.apply(this.scrambleStr);
    this.resetProgress();
  }

  solveSequence(): Move[] {
    const s = this.cube.solve();
    return s ? parseSequence(s) : [];
  }

  hintMove(): Move | null {
    const h = this.cube.hint();
    return h ? parseSequence(h)[0] ?? null : null;
  }

  /** Elapsed milliseconds: 0 before the first move, frozen once solved. */
  elapsed(): number {
    if (this.startTime === null) return 0;
    const end = this.solvedTime ?? performance.now();
    return end - this.startTime;
  }

  private checkSolved(): void {
    if (this.startTime !== null && this.solvedTime === null && this.cube.is_solved()) {
      this.solvedTime = performance.now();
    }
  }

  private resetProgress(): void {
    this.history = [];
    this.redoStack = [];
    this.startTime = null;
    this.solvedTime = null;
  }
}
