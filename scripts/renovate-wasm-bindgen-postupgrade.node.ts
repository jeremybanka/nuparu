import fs from "node:fs";
import { execFileSync } from "node:child_process";

const JS_SYS_PATCH_OFFSET = 23;
const MISE_TOML_PATH = "mise.toml";

function fail(message: string): never {
  console.error(message);
  process.exit(1);
}

function parseVersion(input: string): { major: number; minor: number; patch: number } {
  const match = input.match(/^(\d+)\.(\d+)\.(\d+)$/);

  if (!match) {
    fail(`Expected a semver version like 0.2.122, got: ${input}`);
  }

  const [, major, minor, patch] = match;

  return {
    major: Number(major),
    minor: Number(minor),
    patch: Number(patch),
  };
}

function jsSysVersionForWasmBindgen(version: string): string {
  const parsed = parseVersion(version);

  if (parsed.major !== 0 || parsed.minor !== 2) {
    fail(`Expected a wasm-bindgen 0.2.x version, got: ${version}`);
  }

  if (parsed.patch < JS_SYS_PATCH_OFFSET) {
    fail(`Cannot derive a js-sys version from wasm-bindgen ${version}`);
  }

  return `0.3.${parsed.patch - JS_SYS_PATCH_OFFSET}`;
}

function runCargo(args: string[], dryRun: boolean): void {
  const rendered = ["cargo", ...args].join(" ");

  if (dryRun) {
    console.log(rendered);
    return;
  }

  execFileSync("cargo", args, {
    cwd: process.cwd(),
    stdio: "inherit",
  });
}

function updatePinnedCliVersion(
  currentVersion: string,
  nextVersion: string,
  dryRun: boolean,
): void {
  const before = fs.readFileSync(MISE_TOML_PATH, "utf8");
  const currentLine = `"cargo:wasm-bindgen-cli" = "${currentVersion}"`;
  const nextLine = `"cargo:wasm-bindgen-cli" = "${nextVersion}"`;

  if (before.includes(nextLine)) {
    console.log(`wasm-bindgen-cli already pinned to ${nextVersion}.`);
    return;
  }

  if (!before.includes(currentLine)) {
    fail(`Could not find ${currentLine} in ${MISE_TOML_PATH}`);
  }

  if (dryRun) {
    console.log(`update ${MISE_TOML_PATH}: ${currentLine} -> ${nextLine}`);
    return;
  }

  fs.writeFileSync(MISE_TOML_PATH, before.replace(currentLine, nextLine));
  console.log(`Pinned wasm-bindgen-cli ${nextVersion} in ${MISE_TOML_PATH}.`);
}

const [currentVersion, nextVersion, maybeDryRun] = process.argv.slice(2);
const dryRun = maybeDryRun === "--dry-run";

if (!currentVersion || !nextVersion) {
  fail(
    "Usage: node ./scripts/renovate-wasm-bindgen-postupgrade.node.ts <current-version> <next-version> [--dry-run]",
  );
}

const jsSysVersion = jsSysVersionForWasmBindgen(nextVersion);

console.log(`Aligning js-sys ${jsSysVersion} with wasm-bindgen ${nextVersion}.`);

updatePinnedCliVersion(currentVersion, nextVersion, dryRun);

runCargo(
  ["update", "--config", "net.git-fetch-with-cli=true", "-p", "js-sys", "--precise", jsSysVersion],
  dryRun,
);
