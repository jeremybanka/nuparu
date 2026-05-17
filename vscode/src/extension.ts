import cp from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext): void {
  const provider: vscode.DocumentFormattingEditProvider = {
    provideDocumentFormattingEdits(document) {
      return runFormatter(document);
    },
  };

  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider(
      [{ language: "nushell" }],
      provider
    )
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("nuparu.formatDocument", async () => {
      const editor = vscode.window.activeTextEditor;

      if (!editor) {
        void vscode.window.showInformationMessage(
          "nuparu: No active editor to format."
        );
        return;
      }

      if (editor.document.languageId !== "nushell") {
        void vscode.window.showInformationMessage(
          "nuparu: The active editor is not a Nushell file."
        );
        return;
      }

      try {
        const edits = await runFormatter(editor.document);
        const applied = await editor.edit((editBuilder) => {
          for (const edit of edits) {
            editBuilder.replace(edit.range, edit.newText);
          }
        });

        if (!applied) {
          void vscode.window.showWarningMessage(
            "nuparu: VS Code could not apply the formatting edits."
          );
        }
      } catch (error) {
        void vscode.window.showErrorMessage(
          error instanceof Error ? `nuparu: ${error.message}` : "nuparu failed."
        );
      }
    })
  );
}

async function runFormatter(
  document: vscode.TextDocument
): Promise<vscode.TextEdit[]> {
  const config = vscode.workspace.getConfiguration("nuparu", document.uri);
  const command = resolveFormatterPath(document, config.get<string>("path", ""));
  const extraArgs = config.get<string[]>("extraArgs", []);
  const cwd = workspaceFolderPath(document.uri);

  const formatted = await new Promise<string>((resolve, reject) => {
    const child = cp.spawn(command, extraArgs, {
      cwd,
      stdio: ["pipe", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");

    child.stdout.on("data", (chunk: string) => {
      stdout += chunk;
    });

    child.stderr.on("data", (chunk: string) => {
      stderr += chunk;
    });

    child.on("error", (error) => {
      reject(
        new Error(`Failed to start nuparu (${command}). ${error.message}`)
      );
    });

    child.on("close", (code) => {
      if (code === 0) {
        resolve(stdout);
      } else {
        reject(new Error(stderr.trim() || `nuparu exited with status ${code}.`));
      }
    });

    child.stdin.end(document.getText());
  });

  return [vscode.TextEdit.replace(fullDocumentRange(document), formatted)];
}

function workspaceFolderPath(uri: vscode.Uri): string | undefined {
  const folder = vscode.workspace.getWorkspaceFolder(uri);
  return folder?.uri.fsPath;
}

function fullDocumentRange(document: vscode.TextDocument): vscode.Range {
  const start = new vscode.Position(0, 0);
  const lastLine = document.lineCount === 0 ? 0 : document.lineCount - 1;
  const end = document.lineAt(lastLine).range.end;
  return new vscode.Range(start, end);
}

export function deactivate(): void {}

function resolveFormatterPath(
  document: vscode.TextDocument,
  configuredPath: string
): string {
  const trimmedPath = configuredPath.trim();
  const candidates = new Set<string>();

  if (trimmedPath.length > 0) {
    candidates.add(resolveConfiguredPath(document, trimmedPath));
  }

  for (const candidate of pathCandidatesFromEnv("nuparu")) {
    candidates.add(candidate);
  }

  candidates.add(path.join(os.homedir(), ".cargo", "bin", "nuparu"));
  candidates.add(path.join(os.homedir(), ".local", "bin", "nuparu"));

  for (const folder of vscode.workspace.workspaceFolders ?? []) {
    candidates.add(path.join(folder.uri.fsPath, "target", "debug", "nuparu"));
    candidates.add(path.join(folder.uri.fsPath, "target", "release", "nuparu"));
  }

  for (const candidate of candidates) {
    if (isExecutable(candidate)) {
      return candidate;
    }
  }

  throw new Error(
    [
      "Could not find the nuparu executable.",
      "Set `nuparu.path` in settings or install `nuparu` into a common location such as `~/.cargo/bin/nuparu`.",
      "Searched:",
      ...Array.from(candidates).map((candidate) => `- ${candidate}`),
    ].join("\n")
  );
}

function resolveConfiguredPath(
  document: vscode.TextDocument,
  configuredPath: string
): string {
  if (path.isAbsolute(configuredPath)) {
    return configuredPath;
  }

  const folder = vscode.workspace.getWorkspaceFolder(document.uri);
  return folder ? path.join(folder.uri.fsPath, configuredPath) : configuredPath;
}

function pathCandidatesFromEnv(commandName: string): string[] {
  const pathValue = process.env.PATH ?? "";
  return pathValue
    .split(path.delimiter)
    .filter(Boolean)
    .map((entry) => path.join(entry, commandName));
}

function isExecutable(filePath: string): boolean {
  try {
    fs.accessSync(filePath, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}
