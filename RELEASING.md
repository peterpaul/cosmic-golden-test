# Releasing

Releases are managed with [`cargo-release`](https://github.com/crate-ci/cargo-release).
It handles the version bump, CHANGELOG update, git commit, tag, and push in one command.

## Prerequisites

```bash
cargo install cargo-release
```

## Workflow

### 1. Update the `[Unreleased]` section

Add all notable changes since the last release under `## [Unreleased]` in
`CHANGELOG.md`. `cargo-release` will rename this section and update the
comparison links automatically — you only need to make sure the content is
accurate and complete before running it.

### 2. Dry run

Always preview the release first. `cargo-release` will show every file it
intends to modify without touching anything:

```bash
cargo release minor --workspace
```

Replace `minor` with `patch` or `major` as appropriate, or pass an explicit
version (`0.2.0`). Check that:

- the version bump in `Cargo.toml` is correct
- `CHANGELOG.md` shows the new version heading and correct date
- the comparison links at the bottom of `CHANGELOG.md` are correct
- the git commit message and tag look right

### 3. Execute

When the dry run looks good, add `--execute` to apply the changes:

```bash
cargo release minor --workspace --execute
```

`cargo-release` will:

1. Bump the version in `Cargo.toml` (and `golden-macros/Cargo.toml`).
2. Replace `## [Unreleased]` in `CHANGELOG.md` with the new versioned heading
   and insert a fresh `## [Unreleased]` above it.
3. Update the comparison links at the bottom of `CHANGELOG.md`.
4. Commit all changes with message `chore: release {{version}}`.
5. Create a git tag (`{{version}}`, no `v` prefix).
6. Push the commit and tag to the remote.

Publishing to crates.io is disabled because the crate depends on `libcosmic`
via a git URL, which crates.io does not allow.

### 4. Create a GitHub Release

After the tag is pushed, create a GitHub Release from it:

```bash
gh release create 0.2.0 --title "0.2.0" --notes-file <(
  # Extract the section for this version from CHANGELOG.md
  awk '/^## \[0\.2\.0\]/{found=1; next} found && /^## \[/{exit} found' CHANGELOG.md
)
```

Or use the GitHub web UI: go to **Releases → Draft a new release**, select the
tag, and paste the relevant `CHANGELOG.md` section as the description.
