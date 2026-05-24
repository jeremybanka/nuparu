import fs from "node:fs";
import { spawnSync } from "node:child_process";

const commands = [
  ["cargo", ["publish", "-p", "nuparu-core"]],
  ["cargo", ["publish", "-p", "nuparu-cli"]],
];

if (fs.existsSync("packages")) {
  commands.push(["pnpm", ["publish", "-r", "--filter", "./packages/*"]]);
}

if (fs.existsSync("vscode/nuparu-vscode/package.json")) {
  commands.push(["pnpm", ["--filter", "nuparu-vscode", "package"]]);
  commands.push(["pnpm", ["--filter", "nuparu-vscode", "exec", "vsce", "publish"]]);
}

if (fs.existsSync("scripts/publish-dprint.sh")) {
  commands.push(["./scripts/publish-dprint.sh", []]);
}

for (const [command, args] of commands) {
  run(command, args);
}

function run(command, args) {
  console.log(`> ${command} ${args.join(" ")}`.trim());
  const result = spawnSync(command, args, { stdio: "inherit" });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
