set shell := ["zsh", "-cu"]

default:
  @just --list

changeset:
  pnpm exec changeset

check:
  cargo check --workspace

scripts-check:
  pnpm exec tsc -p tsconfig.json

test:
  cargo test --workspace

fmt:
  cargo fmt --all

clippy:
  cargo clippy --workspace --all-targets --all-features -- -D warnings

run:
  cargo run -p nuparu-cli --bin nuparu

version:
  just version-ecma
  just version-crates

version-ecma:
  pnpm exec changeset version

version-crates:
  node ./scripts/version-crates.ts

publish:
  just publish-crates
  just publish-npmjs
  just publish-vsce
  just publish-dprint

publish-crates:
  cargo publish -p nuparu-core
  cargo publish -p nuparu-cli

publish-npmjs:
  pnpm publish -r --filter "./packages/*"

publish-vsce:
  just vscode-build
  pnpm --filter nuparu-vscode exec vsce publish

publish-dprint:
  echo "nuparu-dprint publish is not wired yet; skipping."

vscode-build:
  pnpm --filter nuparu-vscode build

vscode-package:
  pnpm --filter nuparu-vscode package

vscode-install:
  pnpm --filter nuparu-vscode install:codium
