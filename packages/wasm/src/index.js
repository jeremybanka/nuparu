import fs from "node:fs";
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();
let cachedExports;
export function formatText(fileText, options = {}) {
  const exports = getExports();
  const config = {
    indentWidth: options.indentWidth ?? 2,
    maxBlankLines: options.maxBlankLines ?? 1,
    lineWidth: options.lineWidth ?? 80,
  };
  const inputBytes = textEncoder.encode(fileText);
  const inputPtr = exports.nuparu_alloc(inputBytes.length);
  try {
    memoryView(exports).set(inputBytes, inputPtr);
    const outputPtr = exports.nuparu_format(
      inputPtr,
      inputBytes.length,
      config.indentWidth,
      config.maxBlankLines,
      config.lineWidth,
    );
    const outputLen = exports.nuparu_last_output_len();
    try {
      const outputBytes = memoryView(exports).slice(outputPtr, outputPtr + outputLen);
      return textDecoder.decode(outputBytes);
    } finally {
      exports.nuparu_free(outputPtr, outputLen);
    }
  } finally {
    exports.nuparu_free(inputPtr, inputBytes.length);
  }
}
function getExports() {
  if (cachedExports) {
    return cachedExports;
  }
  const bytes = fs.readFileSync(new URL("./nuparu_wasm.wasm", import.meta.url));
  const module = new WebAssembly.Module(bytes);
  const instance = new WebAssembly.Instance(module, {});
  cachedExports = instance.exports;
  return cachedExports;
}
function memoryView(exports) {
  return new Uint8Array(exports.memory.buffer);
}
