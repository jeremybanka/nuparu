import { defineConfig } from "vite-plus";

export default defineConfig({
  run: {
    tasks: {
      "rust-build": {
        command:
          "mise exec rust -- cargo build -p nuparu-wasm --target wasm32-unknown-unknown --release --target-dir target",
        input: [
          { auto: true },
          "!target/**",
          { pattern: "Cargo.lock", base: "workspace" },
          { pattern: "Cargo.toml", base: "workspace" },
          { pattern: "crates/nuparu-core/**", base: "workspace" },
          { pattern: "crates/nuparu-wasm/**", base: "workspace" },
        ],
        output: ["target/**"],
      },
      "bindings-build": {
        command:
          "mise exec cargo:wasm-bindgen-cli -- wasm-bindgen target/wasm32-unknown-unknown/release/nuparu_wasm.wasm --out-dir dist --target web --no-typescript",
        input: [{ auto: true }, "!dist/**"],
        output: ["dist/nuparu_wasm.js", "dist/nuparu_wasm_bg.wasm"],
      },
      "typescript-build": {
        command: "tsc -p tsconfig.build.json",
        input: [{ auto: true }, "!dist/**"],
        output: ["dist/index.d.ts", "dist/index.js"],
      },
      "build-task": {
        command: "vp run rust-build && vp run bindings-build && vp run typescript-build",
      },
    },
  },
});
