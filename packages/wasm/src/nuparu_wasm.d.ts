export function formatText(
  fileText: string,
  indentWidth: number,
  maxBlankLines: number,
  lineWidth: number,
): string;

export function initSync(
  module: ArrayBuffer | ArrayBufferView | WebAssembly.Module,
): WebAssembly.Exports;

export default function init(
  moduleOrPath?: string | URL | Request | Response | WebAssembly.Module | BufferSource,
): Promise<WebAssembly.Exports>;
