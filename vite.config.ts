import { defineConfig } from "vite-plus";

export default defineConfig({
  fmt: {
    ignorePatterns: ["**/node_modules/**", "docs/**", "fixtures/**", "target/**", "**/dist/**"],
  },
  lint: {
    ignorePatterns: ["**/node_modules/**", "docs/**", "fixtures/**", "target/**", "**/dist/**"],
    options: {
      typeAware: true,
      typeCheck: true,
    },
  },
  run: {
    cache: true,
  },
});
