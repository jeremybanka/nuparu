# nufmt VS Code extension

This extension formats `.nu` files by running `nufmt` over stdin and replacing
the whole document with the formatted output.

You can use it either through the normal editor formatting flow or through the
Command Palette command `nufmt: Format Current Nushell File`.

## Development

Open this `vscode/` folder in VS Code and press `F5` to launch an Extension
Development Host.

Build the extension with:

```bash
pnpm install
pnpm build
```

Package a VSIX with:

```bash
pnpm package
```

## Settings

- `nufmt.path`: path to the `nufmt` executable
- `nufmt.extraArgs`: extra command-line arguments to pass to `nufmt`

## Current scope

- whole-document formatting only
- no range formatting yet
- expects `nufmt` to already be installed or built locally
