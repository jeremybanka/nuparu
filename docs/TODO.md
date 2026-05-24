# TODO

## Editor Integration

### VS Code extension

- Package `nuparu` as a formatter command that can run on save and on explicit format requests.
- Register Nushell files (`.nu`) with the formatter contribution so it can be selected as the default formatter.
- Keep the extension aligned with the user-installed `nuparu` binary model and document the expected install paths.
- Add range-formatting support later only if the formatter can guarantee safe partial formatting.

### Helix

- Keep the project-local [.helix/languages.toml](/Users/jem/dojo/nufmt/.helix/languages.toml:1) working with `nuparu` on stdin/stdout.
- Decide whether to add a `--check` mode for CI and editor-adjacent workflows.
- Add a small documented troubleshooting section for PATH issues if Helix cannot find `nuparu`.

### Zed extension

- Evaluate whether Zed should use a lightweight extension wrapper or a language-server-style integration point.
- Register `nuparu` as the formatter for Nushell buffers and wire it into format-on-save.
- Reuse the system-installed `nuparu` discovery model used by the VS Code extension.
- Add a small integration test matrix for macOS and Linux before publishing.
