import childProcess from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const packageDir = path.dirname(import.meta.dirname);
const packageJsonPath = path.join(packageDir, "package.json");

if (!fs.existsSync(packageJsonPath)) {
  throw new Error(`Missing VS Code extension manifest: ${packageJsonPath}`);
}

const manifest = JSON.parse(fs.readFileSync(packageJsonPath, "utf8")) as {
  name: string;
  version: string;
};

const vsixPath = path.join(packageDir, "vsix", `${manifest.name}-${manifest.version}.vsix`);

childProcess.execFileSync("code", ["--install-extension", vsixPath], {
  cwd: packageDir,
  stdio: "inherit",
});
