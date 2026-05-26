# nuparu VS Code extension

This extension formats `.nu` files by running `nuparu` over stdin and replacing
the whole document with the formatted output.

You can use it either through the normal editor formatting flow or through the
Command Palette command `nuparu: Format Current Nushell File`.

## Development

Open the `vscode/nuparu-vscode/` folder in VS Code and press `F5` to launch an
Extension Development Host.

Build the extension with:

```bash
pnpm install
vp run build
```

That build also produces the installable VSIX. If you want the explicit packaging command:

```bash
pnpm run package
```

Install the packaged extension locally with:

```bash
pnpm run vscode:install
```

## Settings

- `nuparu.path`: path to the `nuparu` executable
- `nuparu.extraArgs`: extra command-line arguments to pass to `nuparu`

## Current scope

- whole-document formatting only
- no range formatting yet
- expects `nuparu` to already be installed or built locally
