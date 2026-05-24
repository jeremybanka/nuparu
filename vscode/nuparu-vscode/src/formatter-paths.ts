import os from "node:os";
import path from "node:path";

export interface FormatterCandidateOptions {
  commandName: string;
  configuredPath: string;
  envPath?: string;
  homeDirectory?: string;
  workspaceFolderPaths?: string[];
}

export function collectFormatterCandidates({
  commandName,
  configuredPath,
  envPath = process.env["PATH"] ?? "",
  homeDirectory = os.homedir(),
  workspaceFolderPaths = [],
}: FormatterCandidateOptions): string[] {
  const candidates = new Set<string>();

  if (configuredPath.length > 0) {
    candidates.add(configuredPath);
  }

  for (const candidate of pathCandidatesFromEnv(commandName, envPath)) {
    candidates.add(candidate);
  }

  candidates.add(path.join(homeDirectory, ".cargo", "bin", commandName));
  candidates.add(path.join(homeDirectory, ".local", "bin", commandName));

  for (const workspaceFolderPath of workspaceFolderPaths) {
    candidates.add(path.join(workspaceFolderPath, "target", "debug", commandName));
    candidates.add(path.join(workspaceFolderPath, "target", "release", commandName));
  }

  return Array.from(candidates);
}

export function resolveConfiguredPath(
  configuredPath: string,
  workspaceFolderPath?: string,
): string {
  if (path.isAbsolute(configuredPath)) {
    return configuredPath;
  }

  return workspaceFolderPath ? path.join(workspaceFolderPath, configuredPath) : configuredPath;
}

export function pathCandidatesFromEnv(commandName: string, envPath: string): string[] {
  return envPath
    .split(path.delimiter)
    .filter(Boolean)
    .map((entry) => path.join(entry, commandName));
}
