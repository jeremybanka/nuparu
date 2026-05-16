# nufmt

`nufmt` is a Nushell formatter written in Rust.

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
- exposes a CLI binary named `nufmt`
- includes a `dprint`-compatible process-plugin-shaped entrypoint as a future
  integration path

## Tooling

This repo uses:

- `mise` for toolchain management
- `just` for common project tasks
- Rust for the formatter binary
- a TypeScript + `tsdown` VS Code extension in [`vscode/`](/Users/jem/dojo/nufmt/vscode)

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

VS Code extension tasks:

```bash
just vscode-build
just vscode-package
just vscode-install
```

## Running the formatter

Quick stdin/stdout usage:

```bash
printf 'def greet [] {\nprint "hi"\n}\n' | cargo run --quiet
```

Install the binary to your user Cargo bin directory:

```bash
cargo install --path . --force
```

Then format through the installed executable:

```bash
cat script.nu | ~/.cargo/bin/nufmt
```

## Parser strategy

`nufmt` intentionally builds on Nushell's own crates:

- `nu-parser`
- `nu-protocol`

The main note from that work: the dependency wiring has to keep feature flags
aligned with Nushell's crate graph. The evaluation notes live in
[docs/nushell-parser-evaluation.md](/Users/jem/dojo/nufmt/docs/nushell-parser-evaluation.md:1).

## Editor integration

There is already a VS Code-compatible extension under
[vscode/README.md](/Users/jem/dojo/nufmt/vscode/README.md:1). It can:

- format the current Nushell document
- participate in normal editor formatting
- discover `nufmt` automatically from common install locations such as
  `~/.cargo/bin/nufmt`

Planned editor follow-up work is tracked in
[docs/TODO.md](/Users/jem/dojo/nufmt/docs/TODO.md:1), including:

- VS Code polish and publishing
- Helix integration
- Zed integration

## Documentation

Primary-source Nushell and `dprint` references are cached in
[docs/README.md](/Users/jem/dojo/nufmt/docs/README.md:1).

## Current status

The formatter is in the "real and useful, but still early" stage:

- it has fixture coverage based on real Nushell scripts
- it avoids several semantic bugs the original prototype had
- it still needs broader formatting policy decisions and more fixture/snapshot
  coverage before calling it stable
