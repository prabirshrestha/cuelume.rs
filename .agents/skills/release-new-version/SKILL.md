---
name: release-new-version
description: Release a new version of the cuelume Rust crate by pulling main, checking CI, choosing and applying a semantic version bump, refreshing Cargo.lock, validating the native and WASM package, pushing a conventional release commit, waiting for CI, then tagging to trigger crates.io Trusted Publishing. Use for explicit invocations like "/release-new-version" or requests to release, version, tag, or publish cuelume.
---

# Release New Version

## Preconditions

- Work from the repository root and on `main` only.
- Treat the release as live unless the user requests a dry run.
- Inspect `git status --short --branch`. Stop before mixing a release with user changes.
- Pull with `git pull --ff-only` when `main` tracks a remote.
- Do not run `cargo publish` locally. A matching tag starts publishing in GitHub Actions.
- Confirm that `cuelume` already exists on crates.io. The first publication must
  be done manually with an API token.
- The crate already has a crates.io GitHub Trusted Publisher for repository
  `prabirshrestha/cuelume.rs` and workflow `ci.yaml`. Treat this external
  configuration as an established repository precondition. Do not ask the user
  to confirm it or try to verify it during a normal release. If trusted
  publishing fails, diagnose the configuration before retrying the tag job.

## Check Upstream CI

After pulling, inspect recent runs:

```sh
gh run list --branch main --limit 5 --json databaseId,headSha,status,conclusion,workflowName,displayTitle,createdAt,url
```

Confirm the latest `CI` run for `HEAD` completed successfully. If this cannot
be confirmed, run all validation commands below before the release commit.

## Bump The Version

1. Read `package.version` from `Cargo.toml`.
2. Inspect release tags and commits since the latest tag:

```sh
git tag --sort=-version:refname | head
git log --oneline <latest-tag>..HEAD
```

3. Use the version requested by the user. Otherwise, select the SemVer bump
   from commit impact. Do not infer a breaking release from a major version of
   zero without checking the changes.
4. Update the version in `Cargo.toml`.
5. Refresh and verify `Cargo.lock`:

```sh
cargo build --all-targets --all-features
cargo build --all-targets --all-features --locked
```

6. Confirm that only `Cargo.toml` and the root package entry in `Cargo.lock`
   changed.

## Validate

Run the complete CI-equivalent suite when upstream CI was not confirmed green
or when the release contains more than the version bump:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo clippy --target wasm32-unknown-unknown --lib --all-features --locked -- -D warnings
cargo build --target wasm32-unknown-unknown --example wasm-app --features wasm-example --locked
cargo package --locked
```

The `wasm32-unknown-unknown` target must be installed. On Linux, native checks
also require the ALSA development package.

## Commit And Publish

1. Commit the bump as `chore: release vX.Y.Z`.
2. Push the commit with `git push origin HEAD`.
3. Wait for the pushed commit's `CI` run to complete successfully:

```sh
gh run watch <run-id> --exit-status
```

4. Create an annotated tag: `git tag -a vX.Y.Z -m "vX.Y.Z"`.
5. Push it: `git push origin vX.Y.Z`.
6. Watch the tag's `CI` run. Its publish job uses GitHub OIDC to request a
   short-lived crates.io token. The repository does not need a `CRATES_TOKEN`
   secret.

Accepted release tag forms are `vX.Y.Z`, `vX.Y.Z-beta.N`, and
`vX.Y.Z-alpha.N`.

Do not push a tag before the release commit CI succeeds. Do not report the
release as published until the tag run's publish step succeeds.

## Report

Report the old and new versions, validation results, release commit, tag,
commit CI result, tag workflow result, and crates.io publication result.
