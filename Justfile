set shell := ["sh", "-eu", "-c"]

default:
    @just --list

# USE FROM SOURCE
i:
    just install
install:
    just install-cargo
    just install-vscode
install-cargo:
    cargo install --path ./crates/nuparu-cli
install-vscode:
    pnpm --filter nuparu-vscode vscode:install

u distribution:
    just use {{ distribution }}
use distribution:
    node ./scripts/nuparu-distribution.node.ts set {{ distribution }}
w:
    just which
which:
    node ./scripts/nuparu-distribution.node.ts which

r:
    just run
run:
    cargo run -p nuparu-cli --bin nuparu

# TEST
t:
    just test
test:
    just test-cargo
    just test-ts
test-cargo:
    cargo test --workspace --all-features
test-ts:
    pnpm exec vp run -r test

# STATIC ANALYSIS
f:
    just fmt
fmt:
    just fmt-cargo
    just fmt-vp
fmt-cargo:
    cargo fmt --all
fmt-cargo-check:
    cargo fmt --all --check
fmt-vp:
    pnpm exec vp fmt
c:
    just check
check:
    just check-clippy
    just check-oxlint
    just check-vp
check-cargo:
    cargo check --workspace --all-features
check-clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings
check-vp:
    pnpm exec vp check

# BUILD SYSTEM
b:
    just build
build:
    just build-cargo
    just build-ts
build-cargo:
    cargo build --workspace
build-ts:
    pnpm exec vp run -r build

# RELEASE SYSTEM
n:
    just notes
notes:
    pnpm exec changeset

# BUMP THE VERSION
version:
    just version-ts
    just version-crates
version-ts:
    pnpm exec changeset version
version-crates:
    node ./scripts/version-crates.node.ts

# SEND TO PUBLISHERS
publish:
    just publish-crates
    just publish-npm
    just publish-vscode
    just publish-dprint
publish-crates:
    cargo publish -p nuparu-core
    cargo publish -p nuparu-cli
publish-npm:
    just build-ts 
    pnpm publish -r --filter "./packages/*"
publish-vscode:
    pnpm --filter nuparu-vscode exec vsce package
    pnpm --filter nuparu-vscode exec vsce publish
publish-dprint:
    echo "nuparu-dprint publish is not wired yet; skipping."
