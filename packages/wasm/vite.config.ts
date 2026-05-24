import { defineConfig } from "vite-plus";

export default defineConfig({
  run: {
    tasks: {
      "build:task": {
        command: "pnpm run build",
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
