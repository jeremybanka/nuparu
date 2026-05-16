# Nushell Parser Evaluation

Date: 2026-05-16

## Recommendation

Use Nushell's own crates as the design target, but do not adopt the current crates.io releases blindly.

The strongest conceptual fit is:

- `nu-parser` for lexing, lite parsing, and full parsing
- `nu-protocol` for AST and parser support types

## Why this looks promising

- `nu-parser` publicly exports `lex`, `lite_parse`, and `parse`.
- `lex` distinguishes Nushell-specific connectors such as `|`, `||`, redirections, comments, semicolons, and end-of-line tokens.
- `lite_parse` groups lexed tokens into commands and pipelines, which is a much better base for formatting than the current line-by-line heuristic.
- `parse` builds Nushell's real AST, which gives us an upgrade path from token-aware formatting to syntax-aware formatting.

## Important caveat

Nushell's parser is type-directed and tied to `StateWorkingSet` / `EngineState`, so the full AST parser is not a tiny standalone lexer crate. That said, the dependency is still reasonable for a formatter because the lower layers we care about are already exposed as public API.

## Feature mismatch finding

The compile failure turned out to be caused by our dependency wiring, not by Nushell's parser crates themselves.

- `nu-parser` depends on `nu-engine` and `nu-protocol` with `default-features = false`.
- I initially added a direct `nu-protocol` dependency with defaults still enabled.
- That enabled `nu-protocol`'s `os` feature globally while `nu-engine` was still compiled through `nu-parser` without `os`.

The result was a cfg split:

- `nu-engine` compiled the `#[cfg(not(feature = "os"))]` initializer for `PipelineExecutionData`.
- `nu-protocol` compiled the `#[cfg(feature = "os")]` struct shape that requires the `exit` field.

That is why the compiler reported a missing `exit` field.

## Working dependency shape

This setup compiles in a standalone verification crate:

```toml
nu-parser = "=0.112.1"
nu-protocol = { version = "=0.112.1", default-features = false }
```

So the current conclusion is stronger than before: the Nushell parser path is viable as long as we keep the feature flags aligned with Nushell's own crate graph.

## Parsing scope note

With the corrected dependency wiring:

- lexing works on the real fixture files
- lite parsing works on the real fixture files
- full AST parsing works for simple inline commands with an empty `EngineState`

However, full parsing of real scripts that use declarations like `use` needs a richer Nushell engine context than `EngineState::new()` alone provides. That is not a blocker for formatter development because lexing and lite parsing already give us a reliable token-aware structural base.

## Current conclusion

This dependency is worth using directly.

The near-term formatter plan should be:

1. Replace the current line-oriented formatter with token + lite-parse aware formatting.
2. Preserve exact token kinds for connectors and comments.
3. Use AST parsing selectively where structure matters, instead of trying to hand-parse Nushell syntax.
