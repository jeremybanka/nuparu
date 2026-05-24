set shell := ["zsh", "-cu"]

default:
    @just --list

# USE FROM SOURCE
i:
    just install
install:
    just install-cargo
    just install-vscode
install-vscode:
    pnpm --filter nuparu-vscode install
install-cargo:
    cargo install --path ./crates/nuparu-cli
u distribution:
    just use distribution={{ distribution }}
use distribution:
    if [ {{ distribution }} == "cargo" ] then just vscode-use-cargo;
    else if [ {{ distribution }} == "npm" ] then just vscode-use-npm;
    else printf '%s\n' 'Invalid distribution. Must be "cargo" or "npm".'; fi; fi
vscode-use-cargo:
    mkdir -p .vscode
    printf '%s\n' cargo > .vscode/nuparu-distribution
    printf '%s\n' 'VS Code nuparu distributable set to cargo.'
vscode-use-npm:
    mkdir -p .vscode
    printf '%s\n' npm > .vscode/nuparu-distribution
    printf '%s\n' 'VS Code nuparu distributable set to npm.'
vscode-which-nuparu:
    if [ -f .vscode/nuparu-distribution ]; then tr -d '[:space:]' < .vscode/nuparu-distribution; else printf '%s' cargo; fi

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
    just fmt-vp
fmt-cargo:
    cargo fmt --all
fmt-vp:
    pnpm exec vp fmt
c:
    just check
check:
    just check-cargo
    just check-clippy
    just check-vp
check-vp:
    pnpm exec vp check
check-cargo:
    cargo check --workspace
check-clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# BUILD SYSTEM
b:
    just build
build:
    just build-cargo
    just build-vp
build-cargo:
    cargo build --workspace
build-ts:
    just build-vp
    just build-vscode
build-vp:
    pnpm exec vp run -r build
build-vscode:
    pnpm --filter nuparu-vscode package

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
    node ./scripts/version-crates.ts

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
    just packages-build
    pnpm publish -r --filter "./packages/*"
publish-vscode:
    just vscode-build
    pnpm --filter nuparu-vscode exec vsce publish
publish-dprint:
    echo "nuparu-dprint publish is not wired yet; skipping."
