import { defineConfig } from "vite-plus";

export default defineConfig({
  run: {
    tasks: {
      "build:task": {
        command:
          "cargo build -p nuparu-wasm --target wasm32-unknown-unknown --release && wasm-bindgen ../../target/wasm32-unknown-unknown/release/nuparu_wasm.wasm --out-dir dist --target web --no-typescript && tsc -p tsconfig.build.json",
        input: [
          { auto: true },
          "!dist/**",
          { pattern: "Cargo.lock", base: "workspace" },
          { pattern: "Cargo.toml", base: "workspace" },
          { pattern: "!target/**", base: "workspace" },
          { pattern: "crates/nuparu-core/**", base: "workspace" },
          { pattern: "crates/nuparu-wasm/**", base: "workspace" },
        ],
        output: ["dist/**"],
      },
    },
  },
});
