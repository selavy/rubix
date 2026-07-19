# Rubix — 3D Rubik's Cube Solver

A web application for solving a 3×3×3 Rubik's Cube in the browser. The user
manipulates a fully 3D cube with mouse, keyboard, or touch; every move is
recorded in standard notation with a live move counter. A **Solve** button
reveals a solution computed with Kociemba's two-phase algorithm, and a
**Hint** button reveals just the next move of that solution.

The performance-critical core (cube state, move engine, scramble generation,
and the solver) is written in **Rust and compiled to WebAssembly**. Rendering,
input, and UI are handled by **TypeScript + Three.js** in the browser.

---

## 1. Goals & Scope

- Render an interactive, photorealistic-enough 3×3×3 cube in 3D.
- Add a button labeled "Scramble" that scrambles the cube.
- Let an anonymous user solve it by hand using mouse, keyboard, or touch.
- Record every move in standard (Singmaster) notation with a running counter,
  displayed on the right side of the screen.
- Detect a win: every face is a single solid color, in any orientation.
    - If a win is detected, show an overlay that says "Complete!" that if the
      user clicks or presses escape goes away.
- Provide **Solve** (show a full short solution) and **Hint** (reveal only the
  next move) powered by Kociemba's two-phase algorithm.
- Support a second **account mode**: logged-in users persist solve history,
  personal bests, and in-progress state across sessions.
- Work on both desktop (mouse + keyboard) and mobile (touch).

### Out of scope (initial release)
- Cube sizes other than 3×3×3 (Kociemba's two-phase is 3×3-specific).
- Multiplayer / real-time races.
- Native mobile apps (the web app is responsive and touch-capable instead).

---

## 2. Modes

### 2.1 Anonymous mode (no login)
The default experience. No account required. The user can:
- Generate a randomized cube.
- Solve it by hand with mouse / keyboard / touch.
- See the move list, move counter, and timer.
- Use **Solve** and **Hint**.

In-progress state may be cached in `localStorage` so a page refresh does not
lose the current attempt, but nothing is sent to a server.

### 2.2 Account mode (login required)
Everything in anonymous mode, plus persistence tied to a user account:
- **Solve history** — each attempt stored as: scramble, ordered move list,
  elapsed time, move count, solved/abandoned flag, timestamp.
- **Personal bests** — fewest moves and fastest time.
- **Resume** — the current in-progress cube state is saved server-side so the
  user can continue on another device.
- **Preferences** — custom keybindings and theme (future-friendly).

The two modes share the same gameplay UI; account mode simply adds
authentication, a history/stats panel, and server persistence.

---

## 3. Architecture

```
┌──────────────────────────── Browser ────────────────────────────┐
│                                                                  │
│  TypeScript + Three.js (frontend)                                │
│   • 3D scene, camera, lighting, cubie meshes, animations         │
│   • Input handling (mouse drag, touch gestures, keyboard)        │
│   • DOM UI: move list, counter, timer, buttons, auth, history    │
│   • Calls into wasm for all cube logic; renders the result       │
│                                                                  │
│            │  wasm-bindgen boundary (typed JS API)               │
│            ▼                                                      │
│  Rust core (compiled to WebAssembly)                             │
│   • Cube state model + move engine (apply moves, query facelets) │
│   • Random-state scramble generation                             │
│   • Win detection                                                │
│   • Kociemba two-phase solver + pruning tables                   │
│   • Move-notation formatting                                     │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
             │  HTTPS (JSON) — account mode only
             ▼
┌──────────────────────── Backend API ────────────────────────────┐
│  Rust (Axum) + PostgreSQL                                        │
│   • Auth (email + password, server sessions)                     │
│   • Solve history, personal bests, saved in-progress state       │
└──────────────────────────────────────────────────────────────────┘
```

### 3.1 Why this split
- The cube model and Kociemba solver are the only compute-heavy, correctness-
  critical pieces — ideal for Rust/wasm, and reusable on the server if needed.
- Three.js is a mature, well-documented WebGL renderer; it gets us a good-
  looking, animated 3D cube far faster than writing our own renderer in wgpu.
- Keeping rendering/UI in TypeScript keeps the DOM, routing, and input glue in
  the ecosystem best suited for it.

### 3.2 Frontend ↔ wasm boundary
The Rust core exposes a small, explicit API via `wasm-bindgen`. The frontend
holds an opaque handle to a `Cube` and never reconstructs cube logic in JS.

Representative interface (illustrative, not final):

```rust
#[wasm_bindgen]
pub struct Cube { /* internal state */ }

#[wasm_bindgen]
impl Cube {
    #[wasm_bindgen(constructor)]
    pub fn solved() -> Cube;

    /// Apply one move in standard notation, e.g. "R", "U'", "F2".
    pub fn apply(&mut self, mv: &str) -> Result<(), JsValue>;

    /// Generate a uniform random *state* scramble; returns the setup
    /// move sequence and leaves the cube in the scrambled state.
    pub fn scramble(&mut self, seed: Option<u64>) -> String;

    /// 54 facelet colors (6 faces × 9), for the renderer to paint cubies.
    pub fn facelets(&self) -> Vec<u8>;

    /// True when every face is a single color (any orientation).
    pub fn is_solved(&self) -> bool;

    /// Full short solution from the current state (space-separated notation).
    pub fn solve(&self) -> Result<String, JsValue>;

    /// Only the first move of `solve()`, for the Hint button.
    pub fn hint(&self) -> Result<String, JsValue>;
}
```

The renderer treats the wasm core as the single source of truth for logical
state; Three.js meshes are a visual projection of `facelets()`.

---

## 4. Cube Model & Move Engine (Rust)

### 4.1 Representation
- Internal state uses a solver-friendly coordinate model (corner
  permutation/orientation + edge permutation/orientation), which is compact and
  is exactly what Kociemba's algorithm consumes.
- A `facelets()` view (54 stickers) is derived on demand for rendering and for
  win detection.

### 4.2 Moves & notation
- **Face turns:** `U D L R F B`, each optionally suffixed with `'`
  (counter-clockwise) or `2` (180°).
- **Wide turns:** `Uw Dw Lw Rw Fw Bw` (two layers).
- **Slice moves:** `M E S`.
- **Whole-cube rotations:** `x y z` (with `'` / `2`).

Rotations and slice/wide moves are supported for input and are echoed into the
move list, but internally they are normalized so state, win detection, and the
solver stay consistent. Whole-cube rotations do not change the solved status.

### 4.3 Move metric
The move counter uses **HTM (Half-Turn Metric)**: each of `U`, `U'`, `U2`
counts as **1 move**. Whole-cube rotations (`x y z`) are shown in the list but
do **not** increment the solve counter (they reorient the view, not the cube).

### 4.4 Scramble generation
Uses a **WCA-style random-state scramble**: a uniformly random *solvable* cube
state is chosen, then the solver produces a setup sequence to reach it. This is
unbiased, unlike applying N random moves. An optional seed enables reproducible
scrambles (useful for shared challenges and tests).

### 4.5 Win detection
The cube is solved when **each of the six faces is a single uniform color**,
evaluated orientation-independently — solving into any orientation counts as a
win.

---

## 5. Solver (Kociemba Two-Phase)

- **Algorithm:** Kociemba's two-phase algorithm, implemented in Rust.
- **Behavior for the Solve button:** return the **first solution** the search
  finds. This is fast (typically well under a second) and usually yields ~21–24
  moves in HTM.
- **Important nuance:** the first two-phase solution is *short*, not provably
  minimal. The UI therefore labels the result as **"a solution,"** not
  "optimal." The architecture leaves room to later swap in (a) a time-budgeted
  search that returns the best solution found within N milliseconds, or (b) a
  true optimal solver — behind the same button, without UI changes.
- **Pruning tables:** the two-phase move/pruning tables are generated at build
  time (or on first load) and shipped with / cached by the wasm module. Table
  size and load strategy are a build concern; the tables are computed once and
  reused for every solve in the session.
- **Hint:** computed as the first move of `solve()` from the current state, so
  hints stay consistent with the displayed full solution and adapt as the user
  makes their own moves.

---

## 6. Rendering & Interaction (Three.js)

### 6.1 Scene
- 26 visible cubies (3×3×3 minus the hidden center) rendered as rounded cubes
  with per-face sticker colors driven by `facelets()`.
- Perspective camera orbiting the cube; soft lighting for depth.
- **Color scheme:** standard Western — white opposite yellow, red opposite
  orange, blue opposite green. Dark UI theme around the canvas.

### 6.2 Move animation
- Every logical move animates: the affected layer rotates 90°/180° over a short
  tween, then state is committed to the wasm core.
- Animations queue so rapid inputs play back smoothly and never desync from the
  logical state (wasm remains the source of truth; the mesh re-syncs to
  `facelets()` after each committed move).

### 6.3 Input

**Mouse / touch**
- **Drag starting on a face:** twists the layer under the pointer. Drag
  direction determines which of the two in-plane axes turns and its sign.
- **Drag starting on empty space:** orbits the camera around the cube.
- **Pinch (touch) / scroll (mouse):** zoom.

**Keyboard**
- Speedcubing-style bindings (in the spirit of cstimer): home-row keys mapped to
  the common face turns, with a modifier/paired key for the inverse, e.g. one
  key for `R` and its mirror for `R'`. A visible **key legend** (toggleable)
  shows the current mapping. Account users can later customize bindings.

All three input methods funnel into the same "apply move" path, so the move
list, counter, and win check behave identically regardless of input.

---

## 7. UI Layout

```
┌───────────────────────────────────────────────────────────────┐
│  Rubix            [Anonymous ▾ / Sign in]           [Theme]     │
├─────────────────────────────────────────┬─────────────────────┤
│                                          │  Moves        (23)  │
│                                          │  ─────────────────  │
│                                          │  1. R    2. U'      │
│              3D CUBE CANVAS              │  3. F2   4. L       │
│           (drag to turn / orbit)         │  5. ...             │
│                                          │  (scrolls)          │
│                                          │                     │
│                                          │  ⏱  00:42.13        │
├─────────────────────────────────────────┴─────────────────────┤
│  [New Scramble] [Reset] [Undo] [Redo]   [Hint]  [Solve]  [⌨]   │
└───────────────────────────────────────────────────────────────┘
```

### 7.1 Components
- **3D canvas** — the interactive cube (center/left).
- **Move panel** (right) — ordered list of moves in notation, with a move
  **counter** badge. Auto-scrolls to the latest move.
- **Timer** — starts on the first move after a scramble, stops on win.
- **Controls bar:**
  - **New Scramble** — generate a fresh random-state scramble.
  - **Reset** — return the cube to the current scramble's starting position and
    clear the move list.
  - **Undo / Redo** — step backward/forward through the user's own moves.
  - **Hint** — reveal the next move; highlight that face with a directional
    arrow on the cube.
  - **Solve** — compute and display the full short solution in the move panel
    (visually distinguished from the user's own moves) and offer **animated
    playback** the user can watch or **step through** move by move.
  - **Keyboard legend toggle** (⌨).
- **Win state** — on solve, the timer stops, the cube does a brief celebration
  animation, and a summary shows move count and time (and, in account mode,
  whether it's a personal best).

### 7.2 Responsive layout
On narrow/mobile viewports the move panel collapses beneath the canvas (or into
a slide-over sheet), controls wrap into a touch-friendly bar, and touch gestures
replace mouse drag. Layout uses relative units and flex/grid so nothing
overflows horizontally.

---

## 8. Backend (Account Mode)

- **Stack:** Rust (**Axum**) HTTP API backed by **PostgreSQL**.
- **Auth:** email + password with server-side sessions (hashed passwords via a
  modern KDF, e.g. Argon2). OAuth can be added later without schema upheaval.
- **Persistence:**
  - `users` — id, email, password hash, created_at.
  - `solves` — id, user_id, scramble, move_list, move_count, elapsed_ms,
    solved (bool), created_at.
  - `saved_state` — user_id → current in-progress cube state + move list, for
    cross-device resume.
  - `preferences` — user_id → keybindings, theme (future).
- **API sketch:** `POST /auth/register`, `POST /auth/login`,
  `POST /auth/logout`, `GET /me`, `GET /solves`, `POST /solves`,
  `GET /saved-state`, `PUT /saved-state`.
- Anonymous mode never calls the backend.

---

## 9. Accessibility

- **Colorblind-friendly option:** an alternate high-contrast sticker palette
  and/or per-sticker symbols, toggleable in settings.
- **Keyboard-only play:** the full game is playable without a pointer; the key
  legend is discoverable and every control is focusable/operable via keyboard.
- Respect `prefers-reduced-motion` by shortening or disabling non-essential
  animations. Sufficient contrast in the surrounding dark UI.

---

## 10. Tech Stack Summary

| Concern            | Choice                                            |
|--------------------|---------------------------------------------------|
| Cube logic, scramble, solver | Rust → WebAssembly (`wasm-bindgen`)     |
| Solver algorithm   | Kociemba two-phase (first solution)               |
| 3D rendering       | Three.js (WebGL)                                  |
| Frontend UI / glue | TypeScript                                        |
| Move metric        | HTM (half-turn)                                   |
| Scramble           | WCA-style random-state                            |
| Backend API        | Rust (Axum)                                       |
| Database           | PostgreSQL                                        |
| Auth               | Email + password, server sessions                 |
| Hosting            | Static frontend on a CDN (e.g. Cloudflare/Netlify Pages); API on a small host (e.g. Fly.io/Railway) |
| Platforms          | Desktop (mouse + keyboard) and mobile (touch)     |

---

## 11. Open Questions / Future Work

- Time-budgeted or fully optimal solve mode behind the same **Solve** button.
- Support for larger cubes (would need a different solving strategy).
- Shared/seeded daily scrambles and leaderboards.
- OAuth login providers.
- Custom keybinding editor UI.
- Sticker symbols for colorblind mode.
