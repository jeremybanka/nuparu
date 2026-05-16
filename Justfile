set shell := ["zsh", "-cu"]

default:
  @just --list

check:
  cargo check

test:
  cargo test

fmt:
  cargo fmt

clippy:
  cargo clippy --all-targets --all-features -- -D warnings

run:
  cargo run

vscode-build:
  pnpm --dir vscode build

vscode-package:
  pnpm --dir vscode package

vscode-install:
  pnpm --dir vscode install:codium
