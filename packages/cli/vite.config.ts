import { defineConfig } from "vite-plus";

export default defineConfig({
  pack: {
    entry: {
      cli: "src/cli.ts",
      index: "src/index.ts",
    },
    dts: true,
    platform: "node",
    sourcemap: true,
    target: "node26",
    unbundle: true,
  },
  test: {
    include: ["__tests__/**/*.test.ts"],
  },
});
