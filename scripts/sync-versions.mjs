import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const publicPackagePaths = [
  "vscode/nuparu-vscode/package.json",
  "packages/nuparu/package.json",
  "packages/nuparu-wasm/package.json",
  "dprint/nuparu-dprint/package.json",
].filter((relativePath) => fs.existsSync(path.join(root, relativePath)));

if (publicPackagePaths.length === 0) {
  throw new Error("No public package manifests were found to source the release version.");
}

const packageVersions = new Map();
for (const relativePath of publicPackagePaths) {
  const manifest = readJson(relativePath);
  packageVersions.set(relativePath, manifest.version);
}

const [sharedVersion] = new Set(packageVersions.values());
if (!sharedVersion) {
  throw new Error("Could not determine a shared version.");
}

for (const [relativePath, version] of packageVersions) {
  if (version !== sharedVersion) {
    throw new Error(
      `Public package versions are out of sync. ${relativePath} has ${version}, expected ${sharedVersion}.`
    );
  }
}

for (const relativePath of publicPackagePaths) {
  const manifest = readJson(relativePath);
  manifest.version = sharedVersion;
  writeJson(relativePath, manifest);
}

syncCargoWorkspaceVersion(sharedVersion);

console.log(`Synchronized shared release version ${sharedVersion}.`);

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function writeJson(relativePath, value) {
  fs.writeFileSync(path.join(root, relativePath), `${JSON.stringify(value, null, 2)}\n`);
}

function syncCargoWorkspaceVersion(version) {
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
}
