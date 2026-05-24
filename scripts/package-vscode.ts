import childProcess from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const packageDir = path.join(workspaceRoot, "vscode/nuparu-vscode");
const packageJsonPath = path.join(packageDir, "package.json");

if (!fs.existsSync(packageJsonPath)) {
  throw new Error(`Missing VS Code extension manifest: ${packageJsonPath}`);
}

const manifest = JSON.parse(fs.readFileSync(packageJsonPath, "utf8")) as {
  name: string;
  version: string;
  files?: string[];
};

const stagingRoot = fs.mkdtempSync(path.join(os.tmpdir(), "nuparu-vscode-package-"));
const stagingDir = path.join(stagingRoot, manifest.name);
fs.mkdirSync(stagingDir, { recursive: true });

for (const relativePath of manifest.files ?? []) {
  const sourcePath = path.join(packageDir, relativePath);
  const destinationPath = path.join(stagingDir, relativePath);

  fs.mkdirSync(path.dirname(destinationPath), { recursive: true });
  fs.cpSync(sourcePath, destinationPath, { recursive: true });
}

const stagedManifest = {
  ...manifest,
};

delete (stagedManifest as { devDependencies?: unknown }).devDependencies;

fs.writeFileSync(
  path.join(stagingDir, "package.json"),
  `${JSON.stringify(stagedManifest, null, 2)}\n`,
);

const vsixPath = path.join(packageDir, `${manifest.name}-${manifest.version}.vsix`);
const vscePath = path.join(packageDir, "node_modules/.bin/vsce");

childProcess.execFileSync(vscePath, ["package", "--allow-missing-repository", "--out", vsixPath], {
  cwd: stagingDir,
  stdio: "inherit",
});
