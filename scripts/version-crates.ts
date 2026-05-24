import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const versionSourcePath = "packages/cli/package.json";

if (!fs.existsSync(path.join(root, versionSourcePath))) {
  throw new Error(`Missing version source manifest: ${versionSourcePath}`);
}

const { version } = JSON.parse(
  fs.readFileSync(path.join(root, versionSourcePath), "utf8")
) as {  version: string; };

const cargoTomlPath = path.join(root, "Cargo.toml");
const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
const next = cargoToml
  .replace(
    /(\[workspace\.package\][\s\S]*?version = ")([^"]+)(")/,
    `$1${version}$3`
  )
  .replace(
    /(nuparu-core = \{ path = "crates\/nuparu-core", version = ")([^"]+)(" \})/,
    `$1${version}$3`
  );

fs.writeFileSync(cargoTomlPath, next);

console.log(`Synchronized shared crate version ${version}.`);
