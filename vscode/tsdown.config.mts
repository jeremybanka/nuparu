import { defineConfig } from "tsdown";

export default defineConfig({
  entry: ["src/extension.ts"],
  outDir: "dist",
  format: "cjs",
  dts: false,
  clean: true,
  unbundle: true,
  external: ["vscode"],
  platform: "node",
  target: "node20",
});
