import fs from "node:fs";
import path from "node:path";

const workspaceRoot = process.cwd();
const distributionFilePath = path.join(workspaceRoot, ".nuparu-distribution");
const validDistributions = new Set(["cargo", "npm"]);

function fail(message: string): never {
  console.error(message);
  process.exit(1);
}

function readDistribution(): string {
  if (!fs.existsSync(distributionFilePath)) {
    return "cargo";
  }

  return fs.readFileSync(distributionFilePath, "utf8").trim() || "cargo";
}

function writeDistribution(distribution: string): void {
  if (!validDistributions.has(distribution)) {
    fail('Invalid distribution. Must be "cargo" or "npm".');
  }

  fs.writeFileSync(distributionFilePath, `${distribution}\n`);
  console.log(`nuparu distributable set to ${distribution}.`);
}

const [command, value] = process.argv.slice(2);

switch (command) {
  case "set":
    if (!value) {
      fail("Missing distribution. Usage: node ./scripts/nuparu-distribution.ts set <cargo|npm>");
    }

    writeDistribution(value);
    break;
  case "which":
    console.log(readDistribution());
    break;
  default:
    fail("Usage: node ./scripts/nuparu-distribution.ts <set|which> [cargo|npm]");
}
