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
});
