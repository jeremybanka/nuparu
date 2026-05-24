#!/usr/bin/env node

import fs from "node:fs";

import { formatText } from "@nuparu/wasm";

const args = process.argv.slice(2);

if (args.length > 0) {
  console.error("@nuparu/cli does not support command-line arguments yet.");
  process.exit(1);
}

const input = fs.readFileSync(0, "utf8");
process.stdout.write(formatText(input));
