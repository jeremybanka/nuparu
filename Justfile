set shell := ["zsh", "-cu"]

default:
  @just --list

changeset:
  pnpm exec changeset

check:
ch:
  just check-cargo
  just check-clippy
  just check-vp
check-vp:
cv:
  vp check
check-cargo:
cc:
  cargo check --workspace
check-clippy:
cl:
  cargo clippy --workspace --all-targets --all-features -- -D warnings

build:
b:
  just build-cargo
  just build-vp
build-cargo:
bc:
  cargo build --workspace
build-ts:
bt:
  just build-vp
  just build-vscode
build-vp:
  vp run -r build
build-vscode:
  pnpm --filter nuparu-vscode package

fmt:
f:
  just fmt-vp
fmt-cargo:
fc:
  cargo fmt --all
fmt-vp:
fv:
  pnpm exec vp fmt

test:
t:
  just test-cargo
  just test-ts
test-cargo:
tc:
  cargo test --workspace --all-features
test-ts:
tt:
  pnpm exec vp run -r test

run:
r:
  cargo run -p nuparu-cli --bin nuparu

version:
  just version-ts
  just version-crates
version-ts:
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

install:
i:
  just install-vscode
vscode-install:
  pnpm --filter nuparu-vscode install:codium
