set shell := ["zsh", "-cu"]

default:
  @just --list

check:
  cargo check --workspace

test:
  cargo test --workspace

fmt:
  cargo fmt --all

clippy:
  cargo clippy --workspace --all-targets --all-features -- -D warnings

run:
  cargo run -p nuparu-cli --bin nuparu

version-sync:
  pnpm version-sync

release:
  pnpm release

vscode-build:
  pnpm --filter nuparu-vscode build

vscode-package:
  pnpm --filter nuparu-vscode package

vscode-install:
  pnpm --filter nuparu-vscode install:codium
