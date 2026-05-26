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
  run: {
    tasks: {
      build: {
        command: "vp pack && node __scripts__/vscode-package.node.ts",
        input: [{ auto: true }, "!dist/**", "!vsix/**"],
        output: ["dist/**", "vsix/**"],
      },
    },
  },
  test: {
    include: ["__tests__/**/*.test.ts"],
  },
});
