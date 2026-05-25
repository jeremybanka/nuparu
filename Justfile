set shell := ["sh", "-eu", "-c"]

default:
    @just --list

# USE FROM SOURCE
i:
    just install
install:
    just install-cargo
    command -v code && just install-vscode
install-cargo:
    cargo install --path ./crates/nuparu-cli
install-vscode:
    just build-vscode
    pnpm -r vscode:install

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
build-vscode:
    pnpm exec vp run --filter "./vscode/*..." build
build-npm:
    pnpm exec vp run --filter "./packages/*..." build

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
publish-crates:
    cargo publish -p nuparu-core
    cargo publish -p nuparu-cli
publish-npm:
    just build-npm
    pnpm -r publish --filter "./packages/*"
publish-vscode:
    just build-vscode
    pnpm -r vscode:publish 
