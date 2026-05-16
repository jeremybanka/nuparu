const cp = require("node:child_process");
const vscode = require("vscode");

function activate(context) {
  const provider = {
    provideDocumentFormattingEdits(document) {
      return runFormatter(document);
    },
  };

  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider(
      [{ language: "nushell", scheme: "file" }, { language: "nushell", scheme: "untitled" }],
      provider
    )
  );
}

async function runFormatter(document) {
  const config = vscode.workspace.getConfiguration("nufmt", document.uri);
  const command = config.get("path", "nufmt");
  const extraArgs = config.get("extraArgs", []);
  const cwd = workspaceFolderPath(document.uri);

  const formatted = await new Promise((resolve, reject) => {
    const child = cp.spawn(command, extraArgs, {
      cwd,
      stdio: ["pipe", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");

    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", (error) => {
      reject(
        new Error(
          `Failed to start nufmt (${command}). ${error.message}`
        )
      );
    });
    child.on("close", (code) => {
      if (code === 0) {
        resolve(stdout);
      } else {
        reject(
          new Error(
            stderr.trim() || `nufmt exited with status ${code}.`
          )
        );
      }
    });

    child.stdin.end(document.getText());
  });

  const fullRange = fullDocumentRange(document);
  return [vscode.TextEdit.replace(fullRange, formatted)];
}

function workspaceFolderPath(uri) {
  const folder = vscode.workspace.getWorkspaceFolder(uri);
  return folder ? folder.uri.fsPath : undefined;
}

function fullDocumentRange(document) {
  const start = new vscode.Position(0, 0);
  const lastLine = document.lineCount === 0 ? 0 : document.lineCount - 1;
  const end = document.lineAt(lastLine).range.end;
  return new vscode.Range(start, end);
}

function deactivate() {}

module.exports = {
  activate,
  deactivate,
};
