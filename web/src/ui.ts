// DOM: move list + counter, timer, control buttons, scramble/solution/hint
// readouts, and the "Complete!" and keyboard-legend overlays.

import { Game } from "./game";
import { moveToString, type Move } from "./moves";

export interface UIHandlers {
  scramble: () => void;
  reset: () => void;
  undo: () => void;
  redo: () => void;
  hint: () => void;
  solve: () => void;
}

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

function formatTime(ms: number): string {
  const total = ms / 1000;
  if (total < 60) return total.toFixed(2);
  const m = Math.floor(total / 60);
  const s = total - m * 60;
  return `${m}:${s.toFixed(2).padStart(5, "0")}`;
}

export class UI {
  private count = $("count");
  private movelist = $("movelist");
  private timer = $("timer");
  private scramble = $("scramble");
  private solutionWrap = $("solutionWrap");
  private solution = $("solution");
  private hintWrap = $("hintWrap");
  private hint = $("hint");
  private overlay = $("overlay");
  private overlayStats = $("overlayStats");
  private legend = $("legend");

  constructor(private game: Game, handlers: UIHandlers) {
    for (const btn of document.querySelectorAll<HTMLButtonElement>("#controls button")) {
      const act = btn.dataset.act as keyof UIHandlers | "legend";
      btn.addEventListener("click", () => {
        if (act === "legend") this.toggleLegend();
        else handlers[act]();
      });
    }

    this.overlay.addEventListener("click", () => this.hideComplete());
    this.legend.addEventListener("click", () => this.legend.setAttribute("hidden", ""));
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        this.hideComplete();
        this.legend.setAttribute("hidden", "");
      } else if (e.key === "Backspace") {
        e.preventDefault();
        handlers.undo();
      }
    });

    // Drive the timer display.
    const loop = () => {
      this.timer.textContent = formatTime(this.game.elapsed());
      requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);
  }

  refresh(): void {
    this.count.textContent = `(${this.game.count})`;
    this.movelist.textContent = "";
    this.game.history.forEach((m, i) => {
      const span = document.createElement("span");
      span.className = "mv";
      span.textContent = moveToString(m);
      span.title = `move ${i + 1}`;
      this.movelist.append(span);
    });
    this.movelist.scrollTop = this.movelist.scrollHeight;
    this.scramble.textContent = this.game.scrambleStr || "—";
    this.setDisabled("undo", !this.game.canUndo);
    this.setDisabled("redo", !this.game.canRedo);
  }

  showSolution(seq: Move[]): void {
    this.solution.textContent = seq.map(moveToString).join(" ");
    this.solutionWrap.removeAttribute("hidden");
  }

  clearSolution(): void {
    this.solutionWrap.setAttribute("hidden", "");
  }

  showHint(m: Move): void {
    this.hint.textContent = moveToString(m);
    this.hintWrap.removeAttribute("hidden");
  }

  clearHint(): void {
    this.hintWrap.setAttribute("hidden", "");
  }

  showComplete(): void {
    this.overlayStats.textContent = `${this.game.count} moves · ${formatTime(this.game.elapsed())}`;
    this.overlay.removeAttribute("hidden");
  }

  hideComplete(): void {
    this.overlay.setAttribute("hidden", "");
  }

  private toggleLegend(): void {
    if (this.legend.hasAttribute("hidden")) this.legend.removeAttribute("hidden");
    else this.legend.setAttribute("hidden", "");
  }

  private setDisabled(act: string, disabled: boolean): void {
    const btn = document.querySelector<HTMLButtonElement>(`#controls button[data-act="${act}"]`);
    if (btn) btn.disabled = disabled;
  }
}
