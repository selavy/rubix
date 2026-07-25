import { defineConfig } from "vite";

export default defineConfig({
  // Top-level await (used to init the wasm module) needs a modern target.
  build: { target: "esnext" },
  // The generated wasm glue loads its .wasm via `import.meta.url`; Vite emits it
  // as an asset automatically.
});
