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
