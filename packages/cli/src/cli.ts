#!/usr/bin/env node

import fs from "node:fs";

import { formatText } from "@nuparu/wasm";
import { runCli } from "./run-cli.js";

const exitCode = runCli(
  process.argv.slice(2),
  {
    readStdin() {
      return fs.readFileSync(0, "utf8");
    },
    writeStdout(text) {
      process.stdout.write(text);
    },
    writeStderr(text) {
      process.stderr.write(text);
    },
    readFile(path) {
      return fs.readFileSync(path, "utf8");
    },
    writeFile(path, text) {
      fs.writeFileSync(path, text);
    },
    getFileMode(path) {
      return fs.statSync(path).mode;
    },
    setFileMode(path, mode) {
      fs.chmodSync(path, mode);
    },
  },
  formatText,
);

if (exitCode !== 0) {
  process.exit(exitCode);
}
