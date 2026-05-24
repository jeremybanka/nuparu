set shell := ["zsh", "-cu"]

default:
  @just --list

changeset:
  pnpm exec changeset

check:
  cargo check --workspace

scripts-check:
  ./node_modules/.bin/tsc -p tsconfig.json

packages-check:
  ./node_modules/.bin/tsc -p packages/wasm/tsconfig.json
  ./node_modules/.bin/tsc -p packages/cli/tsconfig.json

packages-build:
  cargo build -p nuparu-wasm --target wasm32-unknown-unknown --release
  wasm-bindgen target/wasm32-unknown-unknown/release/nuparu_wasm.wasm --out-dir packages/wasm/dist --target web --no-typescript
  ./node_modules/.bin/tsc -p packages/wasm/tsconfig.build.json
  ./node_modules/.bin/tsc -p packages/cli/tsconfig.build.json

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
  just packages-build
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
