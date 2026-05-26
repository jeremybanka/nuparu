import childProcess from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const packageDir = path.dirname(import.meta.dirname);
const packageJsonPath = path.join(packageDir, "package.json");

if (!fs.existsSync(packageJsonPath)) {
  throw new Error(`Missing VS Code extension manifest: ${packageJsonPath}`);
}

const vsixPath = path.join(packageDir, "vsix", `nuparu.vsix`);

childProcess.execFileSync("code", ["--install-extension", vsixPath], {
  cwd: packageDir,
  stdio: "inherit",
});
