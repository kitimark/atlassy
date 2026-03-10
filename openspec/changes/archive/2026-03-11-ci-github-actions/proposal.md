## Why

All testing is manual (`cargo fmt`, `cargo clippy`, `cargo test` run locally). The commit-msg hook enforces conventional commits on the developer's machine but has no server-side counterpart. A push to `main` with broken code or a non-conforming message goes undetected. As the project moves toward PR-based workflow with squash merges, automated quality gates are needed to prevent regressions before merge.

## What Changes

- Add GitHub Actions CI workflow triggered on push to `main` and PRs targeting `main`.
- CI runs format check, clippy lint, full workspace test suite, and commit message validation.
- Commit message validation covers HEAD commit on push and PR title on pull requests (squash merge uses PR title).
- Extract commit validation logic into a shared script used by both the local git hook and CI.
- Slim the existing `.githooks/commit-msg` hook to delegate to the shared script.
- Add `rust-toolchain.toml` to pin the stable channel with `rustfmt` and `clippy` components.
- Configure repository for squash-only merges with PR title as the commit message.
- Enable branch protection requiring the CI check to pass before merge.

## Capabilities

### New Capabilities
- `ci-test-workflow`: GitHub Actions workflow that gates pushes and PRs with format, lint, test, and commit message checks.
- `commit-message-validation`: Shared conventional commit validation script used by both local git hook and CI.
- `toolchain-pinning`: Rust toolchain configuration file for reproducible builds across local and CI environments.

### Modified Capabilities
_(none — no existing spec-level behavior changes)_

## Impact

- New files: `.github/workflows/ci.yml`, `rust-toolchain.toml`, `scripts/validate-commit-msg.sh`.
- Modified files: `.githooks/commit-msg` (slimmed to delegate to shared script).
- Repository settings: squash-only merge enforcement, branch protection rules.
- No changes to any Rust crate source code, build outputs, or runtime behavior.
