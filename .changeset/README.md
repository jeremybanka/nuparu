# Changesets

Use `pnpm changeset` when a pull request introduces a user-facing change.

After merging release work, run `pnpm version-packages` to apply the version
bump to npm manifests and then sync the shared Rust and extension versions.
