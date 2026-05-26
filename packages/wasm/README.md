# @nuparu/wasm

`@nuparu/wasm` ships the WASM-compiled Nuparu formatter for Node consumers.

It currently provides:

- a small synchronous formatting API for Node
- a published copy of the formatter runtime for JavaScript tooling
- the shared runtime that powers `@nuparu/cli`

It does not search for or execute a system-installed `nuparu` binary. That
behavior belongs to editor integrations such as the VS Code extension.
