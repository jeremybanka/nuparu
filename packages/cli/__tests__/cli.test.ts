import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { describe, expect, test } from "vite-plus/test";

import { runCli } from "../src/run-cli.js";

describe("runCli", () => {
  test("formats stdin when no arguments are provided", () => {
    let formattedInput = "";
    let stdout = "";
    let stderr = "";

    const exitCode = runCli(
      [],
      {
        readStdin() {
          return "echo hi";
        },
        writeStdout(text) {
          stdout += text;
        },
        writeStderr(text) {
          stderr += text;
        },
      },
      (input) => {
        formattedInput = input;
        return "formatted";
      },
    );

    expect(exitCode).toBe(0);
    expect(formattedInput).toBe("echo hi");
    expect(stdout).toBe("formatted");
    expect(stderr).toBe("");
  });

  test("rejects unsupported command-line arguments", () => {
    let stdout = "";
    let stderr = "";
    let wasFormatterCalled = false;

    const exitCode = runCli(
      ["--help"],
      {
        readStdin() {
          return "echo hi";
        },
        writeStdout(text) {
          stdout += text;
        },
        writeStderr(text) {
          stderr += text;
        },
      },
      () => {
        wasFormatterCalled = true;
        return "formatted";
      },
    );

    expect(exitCode).toBe(1);
    expect(wasFormatterCalled).toBe(false);
    expect(stdout).toBe("");
    expect(stderr).toContain("does not support command-line arguments");
  });

  test("preserves executable mode when rewriting the real update fixture shape", () => {
    const fixturePath = path.resolve(
      import.meta.dirname,
      "../../../fixtures/dotfiles/scripts/update.nu",
    );
    const fixtureText = fs.readFileSync(fixturePath, "utf8");
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "nuparu-cli-"));
    const tempPath = path.join(tempDir, "update.nu");

    fs.writeFileSync(tempPath, fixtureText);
    fs.chmodSync(tempPath, 0o755);

    let stderr = "";
    const exitCode = runCli(
      ["--write", tempPath],
      {
        readStdin() {
          return "";
        },
        writeStdout() {},
        writeStderr(text) {
          stderr += text;
        },
        readFile(filePath) {
          return fs.readFileSync(filePath, "utf8");
        },
        writeFile(filePath, text) {
          fs.writeFileSync(filePath, text);
        },
        getFileMode(filePath) {
          return fs.statSync(filePath).mode;
        },
        setFileMode(filePath, mode) {
          fs.chmodSync(filePath, mode);
        },
      },
      (input) => `${input}\n`,
    );

    expect(exitCode).toBe(0);
    expect(stderr).toBe("");
    expect(fs.statSync(tempPath).mode & 0o777).toBe(0o755);
  });
});
