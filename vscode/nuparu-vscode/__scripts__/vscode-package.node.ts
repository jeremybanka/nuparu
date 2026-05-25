import childProcess from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const packageDir = path.dirname(import.meta.dirname);
const packageJsonPath = path.join(packageDir, "package.json");

if (!fs.existsSync(packageJsonPath)) {
  throw new Error(`Missing VS Code extension manifest: ${packageJsonPath}`);
}

const { devDependencies: _, ...manifest } = JSON.parse(
  fs.readFileSync(packageJsonPath, "utf8"),
) as {
  name: string;
  version: string;
  files?: string[];
  devDependencies?: unknown;
};
const vsixDir = path.join(packageDir, "vsix");

const stagingRoot = fs.mkdtempSync(path.join(os.tmpdir(), "nuparu-vscode-package-"));
const stagingDir = path.join(stagingRoot, manifest.name);
fs.mkdirSync(stagingDir, { recursive: true });
fs.mkdirSync(vsixDir, { recursive: true });

for (const relativePath of manifest.files ?? []) {
  const sourcePath = path.join(packageDir, relativePath);
  const destinationPath = path.join(stagingDir, relativePath);

  fs.mkdirSync(path.dirname(destinationPath), { recursive: true });
  fs.cpSync(sourcePath, destinationPath, { recursive: true, dereference: true });
}

fs.writeFileSync(path.join(stagingDir, "package.json"), `${JSON.stringify(manifest, null, 2)}\n`);

const vsixPath = path.join(vsixDir, `nuparu.vsix`);
const vscePath = path.join(packageDir, "node_modules/.bin/vsce");

childProcess.execFileSync(vscePath, ["package", "--out", vsixPath], {
  cwd: stagingDir,
  stdio: "inherit",
});
