# Nuparu Release & Packaging Architecture

Nuparu is a multi-target Nushell formatter with one shared Rust formatting
engine and coordinated releases across Rust, npm, VS Code, and dprint
integrations.

The primary product is the native Rust CLI. Editor integrations should prefer
to invoke a user-installed `nuparu` binary rather than bundling formatter
runtime code.

## Product Principles

- one formatter engine
- many delivery targets
- native Rust CLI is the default integration surface
- editor extensions are thin clients over a system-installed `nuparu`
- WASM is an optional integration target, not the basis of editor support
- Changesets orchestrates release metadata
- publishing is delegated to custom scripts
- public artifacts release together under one shared version

## Target Repository Topology

```text
crates/
  nuparu-core          # Rust formatting library
  nuparu-cli           # Rust CLI binary

packages/
  nuparu-wasm          # npm package exposing WASM formatter API
  nuparu               # npm CLI wrapper around nuparu-wasm

vscode/
  nuparu-vscode        # VS Code extension

dprint/
  nuparu-dprint        # dprint plugin package/artifacts
```

## Package Responsibilities

### `nuparu-core`

- canonical formatting engine
- pure Rust library
- shared by:
  - `nuparu-cli`
  - `nuparu-wasm`
  - `nuparu-dprint`

### `nuparu-cli`

- native Rust CLI
- primary end-user formatter distribution
- published to crates.io
- installable via:
  - `cargo install`
  - `mise use cargo:nuparu-cli`
- may optionally ship GitHub release binaries
- defines the stable stdin/stdout contract used by editor integrations

### `nuparu-wasm`

- programmatic npm API
- generated from `nuparu-core`
- intended for:
  - browser/editor experiments
  - programmatic Node usage
  - future web integrations
- not required by the VS Code extension

### `nuparu` (npm CLI)

- lightweight Node wrapper around `nuparu-wasm`
- enables:
  - `npx nuparu`
  - npm/pnpm/bun installs
- secondary delivery path for JavaScript-centric users

### `nuparu-vscode`

- VS Code extension
- thin client that invokes a user-installed `nuparu` binary
- should search:
  - explicit `nuparu.path`
  - workspace-local overrides
  - `PATH`
  - common install locations such as Cargo user bins
  - local development builds during repo work
- should not bundle platform-native formatter binaries
- should not depend on `nuparu-wasm` for normal formatting

### `nuparu-dprint`

- dprint plugin package
- may be delivered as a WASM plugin if the integration is a good fit
- should reuse shared formatting logic from `nuparu-core`

## VS Code Distribution Model

The extension should follow a bring-your-own-binary model.

Benefits:

- keeps the extension lightweight
- avoids bundling or downloading platform-specific binaries
- matches the behavior of many successful formatter extensions
- lets users upgrade `nuparu` independently of the extension
- keeps the runtime contract simple: document text in, formatted text out

User guidance should consistently point to:

- `cargo install`
- `mise use cargo:nuparu-cli`
- explicit `nuparu.path` configuration when auto-discovery is insufficient

The extension should provide clear errors that explain:

- that `nuparu` was not found
- which locations were searched
- how to install it
- how to set `nuparu.path`

## Versioning Philosophy

Use one shared version across all public artifacts:

```text
nuparu-core      0.8.0
nuparu-cli       0.8.0
nuparu-wasm      0.8.0
nuparu           0.8.0
nuparu-vscode    0.8.0
nuparu-dprint    0.8.0
```

Benefits:

- simpler support and debugging
- clearer compatibility guarantees
- cleaner release notes
- easier coordination across ecosystems

## Release Orchestration

Use Changesets as the canonical orchestration layer for:

- release notes
- changelog generation
- coordinated npm version bumps
- release PR automation

Changesets is not the publisher. Custom scripts remain responsible for
publishing each target.

## Release Workflow

### Feature PRs

Developers run:

```bash
pnpm changeset
```

Changesets should describe user-facing changes such as:

- formatter behavior changes
- CLI changes
- extension UX or discovery updates
- npm integration changes
- dprint integration changes

### Version PR

`changesets/action` opens a release PR that:

- updates npm package versions
- generates changelogs
- invokes a custom version sync script

The custom version sync script is responsible for:

- syncing Rust crate versions in `Cargo.toml`
- syncing the VS Code extension version
- syncing the dprint package version
- keeping all public artifacts on one shared version

## Publish Pipeline

After merging the release PR:

```bash
pnpm release
```

That script may internally perform:

```bash
cargo publish -p nuparu-core
cargo publish -p nuparu-cli
pnpm publish -r
vsce publish
./scripts/publish-dprint.sh
```

Optional publish steps:

- create GitHub Releases
- upload standalone CLI binaries
- upload dprint WASM artifacts

## Publish Order

Publishing should respect dependency order:

1. `nuparu-core`
2. `nuparu-cli`
3. npm packages
4. `nuparu-vscode`
5. `nuparu-dprint`

This keeps downstream packages from referencing versions that are not yet
available.

## Migration Plan

Implementation should happen in stages:

1. Split the current root crate into a Rust workspace with `nuparu-core` and
   `nuparu-cli`.
2. Introduce a root `pnpm` workspace and Changesets configuration.
3. Move the VS Code extension into the unified workspace without changing its
   runtime model.
4. Add version sync and publish scripts for Rust, npm, VS Code, and dprint.
5. Add `nuparu-wasm` as an optional integration target.
6. Add `nuparu-dprint` once the desired plugin shape is settled.

## Non-Goals For The Initial Packaging Pass

- bundling native formatter binaries inside the VS Code extension
- downloading formatter binaries on extension install
- making the extension depend on a WASM runtime for standard formatting
- requiring all ecosystem targets to exist before the first coordinated release
