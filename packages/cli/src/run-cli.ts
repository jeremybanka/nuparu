import type { FormatOptions } from "@nuparu/wasm";

export interface CliIO {
  readStdin(): string;
  writeStdout(text: string): void;
  writeStderr(text: string): void;
  readFile?(path: string): string;
  writeFile?(path: string, text: string): void;
  getFileMode?(path: string): number;
  setFileMode?(path: string, mode: number): void;
}

export type FormatText = (fileText: string, options?: FormatOptions) => string;

export function runCli(args: string[], io: CliIO, format: FormatText): number {
  if (args.length === 0) {
    io.writeStdout(format(io.readStdin()));
    return 0;
  }

  if (args[0] === "--write" || args[0] === "-w") {
    const filePaths = args.slice(1);
    if (filePaths.length === 0) {
      io.writeStderr("nuparu --write requires at least one file path.\n");
      return 1;
    }

    if (
      io.readFile == null ||
      io.writeFile == null ||
      io.getFileMode == null ||
      io.setFileMode == null
    ) {
      io.writeStderr("nuparu --write is not available in this runtime.\n");
      return 1;
    }

    for (const filePath of filePaths) {
      const input = io.readFile(filePath);
      const output = format(input);
      if (output === input) {
        continue;
      }

      const mode = io.getFileMode(filePath);
      io.writeFile(filePath, output);
      io.setFileMode(filePath, mode);
    }

    return 0;
  }

  if (args.length > 0) {
    io.writeStderr("@nuparu/cli does not support command-line arguments yet.\n");
    return 1;
  }

  return 0;
}
