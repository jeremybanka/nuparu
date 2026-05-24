import { defineConfig } from "vite-plus";

export default defineConfig({
  pack: {
    clean: true,
    deps: {
      neverBundle: ["vscode"],
    },
    dts: false,
    entry: ["src/extension.ts"],
    format: "cjs",
    outDir: "dist",
    platform: "node",
    target: "node20",
    unbundle: true,
  },
  test: {
    include: ["__tests__/**/*.test.ts"],
  },
});
