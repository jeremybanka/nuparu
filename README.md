# nufmt

`nufmt` is an early Nushell formatter in Rust with a `dprint`-compatible process
plugin entrypoint.

## Current scope

- conservative whitespace normalization for `.nu` files
- indentation based on block delimiters
- pipeline and comma spacing cleanup
- a process-plugin shaped `main` for integration with `dprint`

## Development

Use `mise` to install the toolchain, then drive the project with `just`:

```bash
mise install
just check
just test
```

For quick manual testing:

```bash
printf 'def greet [] {\nprint "hi"\n}\n' | just run
```

## Next steps

1. Replace the line-oriented formatter with a Nushell-aware parser and AST.
2. Add snapshot tests from real `.nu` examples.
3. Publish a plugin schema and `dprint.json` setup example.
