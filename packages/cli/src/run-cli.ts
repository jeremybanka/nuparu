import type { FormatOptions } from "@nuparu/wasm";

export interface CliIO {
  readStdin(): string;
  writeStdout(text: string): void;
  writeStderr(text: string): void;
}

export type FormatText = (fileText: string, options?: FormatOptions) => string;

export function runCli(args: string[], io: CliIO, format: FormatText): number {
  if (args.length > 0) {
    io.writeStderr("@nuparu/cli does not support command-line arguments yet.\n");
    return 1;
  }

  io.writeStdout(format(io.readStdin()));
  return 0;
}
