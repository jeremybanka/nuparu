# Changesets

Use `just changeset` when a pull request introduces a user-facing change.

When preparing a release PR, run `just version` to apply the Changesets bump
and then sync the shared Rust and extension versions.

After merging release work, run `just publish` to ship the current version to
its distribution channels.
