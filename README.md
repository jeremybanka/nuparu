# nuparu

`nuparu` is a Nushell formatter written in Rust.

The project now formats from Nushell's official lexer/parser surface instead of
doing purely line-oriented text rewriting. The current formatter is still
conservative, but it already handles real-world `.nu` scripts much more safely
than the first prototype.

## What it does today

- uses `nu-parser` tokenization as the basis for formatting
- normalizes safe same-line spacing around pipelines, `||`, comments, and
  related operator tokens
- preserves multiline string regions verbatim to avoid mangling embedded shell
  or config text
- trims blank lines at the start and end of blocks
- exposes a CLI binary named `nuparu`
- includes a `dprint`-compatible process-plugin-shaped entrypoint as a future
  integration path

## Tooling

This repo uses:

- `mise` for toolchain management
- `just` for common project tasks
- `pnpm` workspaces with shared catalogs for the TypeScript monorepo
- `vite-plus` to orchestrate TypeScript formatting, linting, builds, and tests
- Rust for the formatter binary
- a TypeScript VS Code extension in
  [`vscode/nuparu-vscode/`](/Users/jem/dojo/nuparu/vscode/nuparu-vscode)

Install the toolchain:

```bash
mise install
```

Common tasks:

```bash
just check
just test
just fmt
just clippy
```

TypeScript-specific workspace tasks:

```bash
just ts-check
just ts-test
just ts-build
just ts-fmt
```

VS Code extension tasks:

```bash
just vscode-build
just vscode-package
just vscode-install
```

## Running the formatter

Quick stdin/stdout usage:

```bash
printf 'def greet [] {\nprint "hi"\n}\n' | cargo run -p nuparu-cli --quiet --bin nuparu
```

Install the binary to your user Cargo bin directory:

```bash
cargo install --path crates/nuparu-cli --force
```

Then format through the installed executable:

```bash
cat script.nu | ~/.cargo/bin/nuparu
```

## Parser strategy

`nuparu` intentionally builds on Nushell's own crates:

- `nu-parser`
- `nu-protocol`

The main note from that work: the dependency wiring has to keep feature flags
aligned with Nushell's crate graph. The evaluation notes live in
[docs/nushell-parser-evaluation.md](/Users/jem/dojo/nufmt/docs/nushell-parser-evaluation.md:1).

## Editor integration

There is already a VS Code-compatible extension under
[vscode/nuparu-vscode/README.md](/Users/jem/dojo/nuparu/vscode/nuparu-vscode/README.md:1). It can:

- format the current Nushell document
- participate in normal editor formatting
- discover `nuparu` automatically from common install locations such as
  `~/.cargo/bin/nuparu`

Planned editor follow-up work is tracked in
[docs/TODO.md](/Users/jem/dojo/nufmt/docs/TODO.md:1), including:

- VS Code polish and publishing
- Zed integration

### Helix

This repo now includes a project-local Helix override in
[.helix/languages.toml](/Users/jem/dojo/nufmt/.helix/languages.toml:1).

It configures the built-in `nu` language to:

- use `nu-lsp` as the language server
- use `nuparu` as the formatter
- enable `auto-format`
- use a `text-width` of `80`

If `nuparu` is installed on your `PATH`, Helix should pick it up automatically
for this workspace. You can verify that with:

```bash
hx --health nu
```

In this repo, that should report both:

- `nu-lsp` configured
- `nuparu` configured as the formatter

## Documentation

Primary-source Nushell and `dprint` references are cached in
[docs/README.md](/Users/jem/dojo/nufmt/docs/README.md:1).

## Current status

The formatter is in the "real and useful, but still early" stage:

- it has fixture coverage based on real Nushell scripts
- it avoids several semantic bugs the original prototype had
- it still needs broader formatting policy decisions and more fixture/snapshot
  coverage before calling it stable
