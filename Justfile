set shell := ["zsh", "-cu"]

default:
  @just --list


# USE FROM SOURCE
i:
install:
  just install-vscode
install-vscode:
  pnpm --filter nuparu-vscode install:codium
install-cargo:
  cargo install --path ./packages/cli
r:
run:
  cargo run -p nuparu-cli --bin nuparu

# TEST
t:
test:
  just test-cargo
  just test-ts
tc:
test-cargo:
  cargo test --workspace --all-features
tt:
test-ts:
  pnpm exec vp run -r test

# STATIC ANALYSIS
f:
fmt:
  just fmt-vp
fc:
fmt-cargo:
  cargo fmt --all
fv:
fmt-vp:
  pnpm exec vp fmt
c:
check:
  just check-cargo
  just check-clippy
  just check-vp
cv:
check-vp:
  pnpm exec vp check
cc:
check-cargo:
  cargo check --workspace
cl:
check-clippy:
  cargo clippy --workspace --all-targets --all-features -- -D warnings

# BUILD SYSTEM
b:
build:
  just build-cargo
  just build-vp
bc:
build-cargo:
  cargo build --workspace
bt:
build-ts:
  just build-vp
  just build-vscode
build-vp:
  pnpm exec vp run -r build
build-vscode:
  pnpm --filter nuparu-vscode package

# RELEASE SYSTEM
n:
notes:
  pnpm exec changeset

# BUMP THE VERSION
version:
  just version-ts
  just version-crates
version-ts:
  pnpm exec changeset version
version-crates:
  node ./scripts/version-crates.ts

# SEND TO PUBLISHERS
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

