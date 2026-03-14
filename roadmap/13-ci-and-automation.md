# CI and Automation

## Objective

Establish automated quality gates that validate code correctness, style, and safety on every push and pull request, preventing regressions before merge.

## Current Baseline

- All testing has been manual: `cargo fmt`, `cargo clippy`, `cargo test` run locally.
- No automated CI pipeline exists.
- 76 tests across 7 binaries (5 crates) run reliably in under 60 seconds.
- All tests are self-contained with no external service dependencies (live runtime tests use dummy credentials and assert on expected failures).

## Phase 1: Test Job (Current)

### Scope

Automated test job on GitHub Actions triggered by push to `main` and pull requests targeting `main`.

### Workflow: `.github/workflows/ci.yml`

**Triggers:**

- Push to `main`
- Pull request targeting `main`

**Job: `test`** (runs on `ubuntu-latest`):

| Step | Command | Purpose |
| --- | --- | --- |
| Checkout | `actions/checkout@v4` | Fetch source |
| Toolchain | `dtolnay/rust-toolchain@stable` | Install Rust (toolchain selected by `@ref`, not by `rust-toolchain.toml`) |
| Cache | `Swatinem/rust-cache@v2` | Cache `target/` and cargo registry by `Cargo.lock` hash |
| Format check | `cargo fmt --all -- --check` | Reject unformatted code |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Reject lint warnings |
| Test | `cargo test --workspace` | Run all tests across workspace |

### Action Version Selection

Versions chosen for stability, community adoption, and broad compatibility (researched 2026-03-07).

**Selected stack:**

| Action | Ref | Latest Patch | Stars | Used By | Contributors | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `actions/checkout` | `@v4` | v4.3.1 | 7.6k | universal | GitHub org | Official GitHub action. LTS line, actively patched. v5/v6 exist but require newer runner versions (>= v2.327.1 / v2.329.0). v4 has no runner constraints. |
| `dtolnay/rust-toolchain` | `@stable` | branch-based (no semver) | 1.5k | 70.4k | 15 | Installs latest stable Rust. Toolchain version is selected by the `@ref`, not by input. Maintained by David Tolnay. |
| `Swatinem/rust-cache` | `@v2` | v2.8.2 | 1.8k | widespread | 20+ | Only active major version. Floating `@v2` tag auto-resolves to latest v2.x patch. Caches `target/` and cargo registry by `Cargo.lock` hash. |

**Alternatives evaluated and rejected:**

| Action | Stars | Used By | Status | Rejection Reason |
| --- | --- | --- | --- | --- |
| `actions-rust-lang/setup-rust-toolchain@v1` | 359 | 8.6k | Active (v1.15.3, 22 contributors) | Bundles `Swatinem/rust-cache` internally and adds problem matchers, but 4x fewer stars and 8x fewer dependents than `dtolnay/rust-toolchain`. Higher complexity for marginal benefit. |
| `actions-rs/toolchain@v1` | 607 | legacy | Archived (Oct 2023) | Dead project. Last release v1.0.6 in Mar 2020. Repository is read-only. |

### Design Decisions

- **Toolchain pinning**: `rust-toolchain.toml` pins `stable` channel with `rustfmt` and `clippy` components. Edition 2024 requires Rust >= 1.85.0; any stable release from 1.85.0 onward satisfies this.
- **TLS backend**: workspace uses `rustls-tls` (not `native-tls`), so no `libssl-dev` or system OpenSSL is needed on the runner.
- **No secrets required**: all integration tests self-supply dummy credentials and assert on expected failure behavior. No Confluence API tokens or sandbox access is needed in CI.
- **Fail-fast ordering**: format and clippy checks run before tests to catch trivial issues early without waiting for full test compilation.
- **Caching**: `Swatinem/rust-cache@v2` caches based on `Cargo.lock` hash, significantly reducing build times on repeat runs.
- **Checkout version conservatism**: v4 chosen over v6 because v6 changes credential persistence behavior and requires a newer Actions runner. v4 remains the safest LTS choice for a project that does not need v6 features.

### Done Criteria

- Workflow file exists at `.github/workflows/ci.yml`.
- `rust-toolchain.toml` exists at repo root.
- All existing tests pass in CI on first run.
- PRs to `main` are gated by the test job.

### Branch Protection Setup

PR gating requires branch protection rules in GitHub repo settings. This is a post-implementation step — the CI workflow must exist and have run at least once so GitHub registers the status check name.

**Sequence:**

1. Push `.github/workflows/ci.yml` and `rust-toolchain.toml` to `main`.
2. Wait for the first CI run to complete (registers the `Test` check name).
3. Enable branch protection via `gh api`:

```bash
gh api --method PUT /repos/{owner}/{repo}/branches/main/protection \
  --input - <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["Test"]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null
}
EOF
```

- `"contexts": ["Test"]` matches the job name in `ci.yml`.
- `"strict": true` requires the branch to be up-to-date before merging.
- `"enforce_admins": false` allows repo admins to bypass during initial setup.
- `"required_pull_request_reviews": null` does not require PR reviews (single-contributor project).
- Requires `gh` authenticated with `repo` scope (verified available).

## Phase 2: Release Pipeline (Planned)

### Scope

Automated release pipeline: version bumping, changelog generation, git tagging, GitHub Release creation, and cross-platform binary distribution.

### Tool Selection

**Selected: `release-plz`** — a Rust-native release automation tool that analyzes conventional commits, determines version bumps, generates changelogs, and creates GitHub Releases.

| Tool | Approach | Fit | Decision |
|------|----------|-----|----------|
| `release-plz` | Fully automated: release PR with version bump + changelog, tag + release on merge | Leverages existing conventional commits, no local tooling required | **Selected** |
| `cargo-release` | Semi-automated: operator runs `cargo release` locally, CI picks up the tag | Ties releases to local machine, requires per-developer setup | Rejected |
| Manual tag + CI | Operator bumps `Cargo.toml`, tags, pushes; CI builds on tag | Simplest but most manual steps per release, no automated changelog | Rejected |
| `workflow_dispatch` | Push-button release from GitHub Actions UI | Custom workflow logic to maintain for version bump step | Rejected |

`release-plz` was chosen because:

- The project already uses conventional commits with validation in CI (`scripts/validate-commit-msg.sh`).
- No local tooling setup required — runs entirely in GitHub Actions.
- No vendor lock-in — the tags, releases, and changelog it creates are standard git/GitHub primitives.
- No `CARGO_REGISTRY_TOKEN` needed since the project does not publish to crates.io.

### Versioning Convention

Atlassy is a CLI binary distributed via GitHub Releases, not a library consumed via `Cargo.toml` dependency. The Cargo semver convention (where the left-most non-zero component is the compatibility boundary) applies to library dependency resolution, not to CLI user-facing version semantics.

For CLI users, version numbers should communicate the significance of changes:

| Commit type | Version bump | Example |
|-------------|-------------|---------|
| `fix:` | Patch | 0.1.0 → 0.1.1 |
| `feat:` | Minor | 0.1.0 → 0.2.0 |
| `feat!:` / `BREAKING CHANGE:` | Minor (pre-1.0) | 0.2.0 → 0.3.0 |

Post-1.0: `feat!:` bumps major instead.

Configured via `features_always_increment_minor = true` in `release-plz.toml`. This deviates from Cargo's library convention (where `feat` bumps patch in 0.x) because for a CLI binary, each feature release should be a visible version increment.

Reference: [Cargo SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html), [Specifying Dependencies — Default Requirements](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#default-requirements).

### Release Workflow

Two GitHub Actions workflows coordinate the release process:

**`release-plz.yml`** — triggered on push to `main`:

1. `release-plz release-pr`: Analyzes commits since the last release tag. If releasable changes exist, creates or updates a PR that bumps `version` in workspace `Cargo.toml` and generates/updates `CHANGELOG.md`.
2. `release-plz release`: After the release PR is merged, creates a git tag (e.g., `v0.2.0`) and a GitHub Release with the changelog as the body.

The release PR goes through the existing CI pipeline (`test` job: fmt-check, clippy, tests) before merge.

**`release-build.yml`** — triggered on tag push matching `v*`:

Builds release binaries for 4 targets and uploads them to the existing GitHub Release.

| Target | Runner | Build Method |
|--------|--------|-------------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Native |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | `cross` (Docker-based cross-compilation) |
| `x86_64-apple-darwin` | `macos-13` | Native |
| `aarch64-apple-darwin` | `macos-latest` (M1) | Native |

Each build produces a tarball: `atlassy-cli-{tag}-{target}.tar.gz`.

### Configuration

**`release-plz.toml`** key decisions:

| Setting | Value | Rationale |
|---------|-------|-----------|
| `publish` | `false` | No crates.io publishing |
| `git_only` | `true` | Version determined from git tags, not cargo registry |
| `release` (workspace) | `false` | Skip all packages by default |
| `release` (atlassy-cli) | `true` | Only the CLI binary produces releases |
| `semver_check` | `false` | CLI binary, no library API surface |
| `features_always_increment_minor` | `true` | `feat` bumps minor in 0.x |
| `changelog_include` | all 4 internal crates | Commits to any crate appear in CLI changelog |

**Files:**

| File | Type | Purpose |
|------|------|---------|
| `release-plz.toml` | New | release-plz configuration |
| `.github/workflows/release-plz.yml` | New | Release PR + tag/release automation |
| `.github/workflows/release-build.yml` | New | Cross-platform binary builds on tag |
| `Makefile` | Modified | Add `build-release` target (`cargo build -p atlassy-cli --release`) |

### Prerequisites

1. **GitHub repo setting**: Enable "Allow GitHub Actions to create and approve pull requests" in Settings → Actions → General.
2. **Workflow permissions**: `release-plz.yml` requires `contents: write` and `pull-requests: write`.
3. **Branch protection**: The existing `test` job remains required for merge. Release PRs created by release-plz pass through it like any other PR.

### Done Criteria

- `release-plz.toml` exists at repo root with configuration described above.
- `.github/workflows/release-plz.yml` runs on push to `main` and creates release PRs.
- `.github/workflows/release-build.yml` triggers on `v*` tags and produces 4-platform binaries.
- `make build-release` produces a release binary at `target/release/atlassy-cli`.
- First release PR is created automatically after merging the implementation.
- Merging the release PR creates a git tag and GitHub Release with changelog body.
- All 4 binary tarballs are attached to the GitHub Release.
- Existing CI (`test` job) is unaffected and gates release PRs.

## Phase 3: Extended Automation (Deferred)

### Scope

Additional CI capabilities beyond test/build/release.

### Candidate Work

- Matrix builds (multiple Rust versions).
- Code coverage reporting (e.g., `cargo-llvm-cov` with Codecov or similar).
- Live Confluence smoke tests in CI (requires GitHub Actions secrets for sandbox credentials).
- Scheduled nightly runs for drift detection.
- Dependency audit (`cargo-deny` or `cargo-audit`).

### Signals to Start

- Phase 2 release pipeline is stable and producing releases.
- Project has multiple contributors or external CI consumers.
- Live smoke tests are needed for regression detection beyond local QA.

## Explicitly Out of Scope

- Deployment or publishing pipelines (no production deployment target exists yet).
- Multi-repo orchestration.
- Custom runner infrastructure.

## Dependencies

- Makefile targets mirror CI steps: `make fmt`, `make lint`, `make test`, `make build`.
- Testing strategy: `10-testing-strategy-and-simulation.md`.
- Execution readiness: `07-execution-readiness.md`.
