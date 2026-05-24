import fs from "node:fs";

import { formatText as wasmFormatText, initSync } from "./nuparu_wasm.js";

const wasmBytes = fs.readFileSync(new URL("./nuparu_wasm_bg.wasm", import.meta.url));

let initialized = false;

export interface FormatOptions {
  indentWidth?: number;
  lineWidth?: number;
  maxBlankLines?: number;
}

export function formatText(fileText: string, options: FormatOptions = {}): string {
  initialize();

  return wasmFormatText(
    fileText,
    options.indentWidth ?? 2,
    options.maxBlankLines ?? 1,
    options.lineWidth ?? 80
  );
}

function initialize() {
  if (initialized) {
    return;
  }

  initSync({ module: wasmBytes });
  initialized = true;
}
