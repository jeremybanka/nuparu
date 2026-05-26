import path from "node:path";

import { describe, expect, test } from "vite-plus/test";

import { collectFormatterCandidates, resolveConfiguredPath } from "../src/formatter-paths.js";

describe("resolveConfiguredPath", () => {
  test("preserves absolute paths", () => {
    expect(resolveConfiguredPath("/tmp/nuparu", "/workspace")).toBe("/tmp/nuparu");
  });

  test("resolves relative paths from the workspace root", () => {
    expect(resolveConfiguredPath("bin/nuparu", "/workspace")).toBe(
      path.join("/workspace", "bin/nuparu"),
    );
  });
});

describe("collectFormatterCandidates", () => {
  test("collects configured, PATH, home, and workspace build candidates", () => {
    const candidates = collectFormatterCandidates({
      commandName: "nuparu",
      configuredPath: "/workspace/bin/nuparu",
      envPath: ["/usr/local/bin", "/opt/homebrew/bin"].join(path.delimiter),
      homeDirectory: "/Users/example",
      workspaceFolderPaths: ["/workspace"],
    });

    expect(candidates).toEqual([
      "/workspace/bin/nuparu",
      path.join("/usr/local/bin", "nuparu"),
      path.join("/opt/homebrew/bin", "nuparu"),
      path.join("/Users/example", ".cargo", "bin", "nuparu"),
      path.join("/Users/example", ".local", "bin", "nuparu"),
      path.join("/workspace", "target", "debug", "nuparu"),
      path.join("/workspace", "target", "release", "nuparu"),
    ]);
  });
});
