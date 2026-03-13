# Releasing atosl

This document describes the release flow for publishing a new `atosl` version to crates.io.

If you want the repository to handle the full flow for you, use:

```bash
./deploy.sh patch
```

You can also pass `minor`, `major`, or an explicit version such as `0.1.16`.

## Prerequisites

- You have push access to `origin`
- You are logged in to crates.io on this machine
- The working tree is clean

Check crates.io authentication:

```bash
cargo login
```

## Release checklist

1. Update the crate version in [Cargo.toml](/Users/eevv/focus/atosl-rs/Cargo.toml).
2. Review [README.md](/Users/eevv/focus/atosl-rs/README.md) for any user-visible changes.
3. Refresh Apple goldens on macOS if symbolication behavior changed.
4. Run the full validation suite.
5. Commit the release change and create a git tag.
6. Run a dry-run publish.
7. Publish to crates.io.
8. Push the commit and tag to GitHub.

## Validation

Run:

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo bench --bench batch_symbolize --no-run
```

If you changed Apple-specific logic and you are on macOS, also run:

```bash
./scripts/refresh_apple_goldens.sh
```

## Release steps

## One-command release

From the repository root:

```bash
./deploy.sh patch
```

Other forms:

```bash
./deploy.sh minor
./deploy.sh major
./deploy.sh 0.1.16
```

The script does all of the following:

- Verifies the working tree is clean
- Bumps the crate version in [Cargo.toml](/Users/eevv/focus/atosl-rs/Cargo.toml)
- Regenerates `Cargo.lock`
- Runs validation
- Creates a release commit and tag
- Runs `cargo publish --dry-run`
- Publishes to crates.io
- Pushes the branch and tag to `origin`

If you prefer to release manually, follow the steps below.

Assume the next version is `0.1.16`.

Update the crate version in [Cargo.toml](/Users/eevv/focus/atosl-rs/Cargo.toml):

```toml
version = "0.1.16"
```

Commit and tag:

```bash
git add Cargo.toml Cargo.lock README.md RELEASING.md
git commit -m "Release v0.1.16"
git tag v0.1.16
```

Dry run the publish:

```bash
cargo publish --dry-run
```

If that succeeds, publish:

```bash
cargo publish
```

Push the release commit and tag:

```bash
git push origin main
git push origin v0.1.16
```

## Post-release checks

- Verify the new version appears on crates.io
- Confirm the git tag exists on GitHub
- Optionally install the published crate into a clean environment:

```bash
cargo install atosl --version 0.1.16
```

## Notes

- crates.io versions are immutable. If you publish the wrong version, you need to release a newer one.
- `cargo publish --dry-run` is the best guardrail against packaging mistakes.
- Keep tags aligned with crate versions using the `vX.Y.Z` format.
