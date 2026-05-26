import { execFileSync } from "node:child_process";

const JS_SYS_PATCH_OFFSET = 23;

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

const [_currentVersion, nextVersion, maybeDryRun] = process.argv.slice(2);
const dryRun = maybeDryRun === "--dry-run";

if (!_currentVersion || !nextVersion) {
  fail(
    "Usage: node ./scripts/renovate-wasm-bindgen-postupgrade.node.ts <current-version> <next-version> [--dry-run]",
  );
}

const jsSysVersion = jsSysVersionForWasmBindgen(nextVersion);

console.log(`Aligning js-sys ${jsSysVersion} with wasm-bindgen ${nextVersion}.`);

runCargo(
  ["update", "--config", "net.git-fetch-with-cli=true", "-p", "js-sys", "--precise", jsSysVersion],
  dryRun,
);
