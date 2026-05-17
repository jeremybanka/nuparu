# TODO

## Editor Integration

### VS Code extension

- Package `nufmt` as a formatter command that can run on save and on explicit format requests.
- Register Nushell files (`.nu`) with the formatter contribution so it can be selected as the default formatter.
- Decide whether the extension should bundle platform binaries, download releases, or delegate to a user-installed `nufmt`.
- Add range-formatting support later only if the formatter can guarantee safe partial formatting.

### Helix

- Keep the project-local [.helix/languages.toml](/Users/jem/dojo/nufmt/.helix/languages.toml:1) working with `nufmt` on stdin/stdout.
- Decide whether to add a `--check` mode for CI and editor-adjacent workflows.
- Add a small documented troubleshooting section for PATH issues if Helix cannot find `nufmt`.

### Zed extension

- Evaluate whether Zed should use a lightweight extension wrapper or a language-server-style integration point.
- Register `nufmt` as the formatter for Nushell buffers and wire it into format-on-save.
- Decide on binary discovery: bundled, downloaded, or system-installed.
- Add a small integration test matrix for macOS and Linux before publishing.
