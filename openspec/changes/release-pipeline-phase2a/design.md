## Context

Atlassy is a Rust CLI workspace at version 0.1.0 with 5 crates sharing a workspace version. Phase 1 CI (`ci.yml`) runs fmt-check, clippy, and tests on push/PR to main using `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, and `Swatinem/rust-cache@v2`. There are no git tags, no CHANGELOG, no LICENSE file, and no release process. The project uses conventional commits enforced by a commit-msg hook and CI validation script.

The repo is hosted at `github.com/kitimark/atlassy`. The Makefile has targets for build, test, lint, fmt, qa-setup, and qa-check. There is no `build-release` target.

## Goals / Non-Goals

**Goals:**

- Automated version bumping and changelog generation from conventional commits.
- Controlled release cadence — operator decides when to cut a release by merging a release PR.
- Cross-platform binary distribution via GitHub Releases (4 targets).
- Local release build target for development and QA use.
- Clear install instructions in README.

**Non-Goals:**

- crates.io publishing (deferred to Phase 2b).
- `cargo install` or `cargo binstall` support (requires crates.io, deferred to Phase 2b).
- Install scripts, Homebrew formula, or other package manager integrations.
- Windows builds.
- Automated QA execution using release binaries (deferred — separate QA execution update).

## Decisions

### D1: release-plz over alternatives

**Choice**: `release-plz` for release automation.

**Alternatives considered**:
- `cargo-release`: Ties releases to the operator's local machine. Requires per-developer tooling setup.
- Manual tag + CI: Simplest to set up but most manual steps per release and no automated changelog.
- `workflow_dispatch`: Push-button from GitHub UI, but requires custom workflow logic for version bumping.

**Rationale**: release-plz leverages the existing conventional commit discipline. It runs entirely in GitHub Actions with no local tooling. It produces standard git primitives (tags, releases) with no vendor lock-in. No `CARGO_REGISTRY_TOKEN` needed since this phase doesn't publish to crates.io.

### D2: `release_always = false` for controlled releases

**Choice**: Set `release_always = false` in `release-plz.toml`.

**Alternatives considered**:
- `release_always = true` (default): Every push to main attempts a release. If multiple feature PRs merge in succession, each triggers a separate release.

**Rationale**: With `false`, feature and fix PRs accumulate in a release PR that release-plz maintains. The operator reviews and merges it when ready. This gives explicit control over release timing. The release job detects the merge by checking if the commit's PR branch starts with `release-plz-`.

### D3: `features_always_increment_minor = true`

**Choice**: `feat:` commits bump minor version (0.1.0 → 0.2.0), not patch.

**Alternatives considered**:
- Cargo's library convention: `feat` bumps patch in 0.x, only breaking changes bump minor.

**Rationale**: Atlassy is a CLI binary, not a library. Nobody depends on `^0.1.0` in a `Cargo.toml`. For CLI users downloading from GitHub Releases, `0.1.0 → 0.2.0` communicates a new feature more clearly than `0.1.0 → 0.1.1`. The Cargo semver convention exists to solve dependency resolution problems that don't apply here.

### D4: All-native build matrix (no cross-compilation)

**Choice**: 4 native builds on architecture-matched GitHub runners.

| Target | Runner |
|--------|--------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` |
| `x86_64-apple-darwin` | `macos-15-intel` |
| `aarch64-apple-darwin` | `macos-latest` |

**Alternatives considered**:
- `cross` (Docker-based cross-compilation) for `aarch64-unknown-linux-gnu`: Adds Docker pull overhead and more complex workflow logic.
- `cargo-zigbuild`: Lighter weight cross-compilation but less battle-tested.

**Rationale**: GitHub now provides native ARM64 Linux runners (`ubuntu-24.04-arm`) and Intel macOS runners (`macos-15-intel`) as standard hosted runners, free on public repos. All 4 targets can use identical build steps (`cargo build -p atlassy-cli --release`). No cross-compilation tooling, no Docker, no special configuration. Note: `macos-13` is no longer listed in the GitHub runner table; `macos-15-intel` is the correct label for Intel macOS.

### D5: `actions/checkout@v4` with `persist-credentials: false`

**Choice**: Use `@v4` in release workflows, matching Phase 1 CI.

**Alternatives considered**:
- `actions/checkout@v6`: Recommended by release-plz quickstart. Defaults to `persist-credentials: false`.

**Rationale**: Consistency with existing CI. All runners (ubuntu-24.04, macos-15) support both versions. Using `@v4` with explicit `persist-credentials: false` and `fetch-depth: 0` achieves the same behavior release-plz requires without diverging from the established CI pattern.

### D6: `gh release upload` over `softprops/action-gh-release`

**Choice**: Use the GitHub CLI (`gh release upload`) to attach binaries to releases.

**Alternatives considered**:
- `softprops/action-gh-release` (6k+ stars): Popular third-party action for managing GitHub Releases.

**Rationale**: `gh` is pre-installed on every GitHub runner. No additional action to version-pin or track. Consistent with Phase 1 philosophy of minimizing third-party action dependencies. `gh release upload` takes a tag name and file glob — simple and reliable.

### D7: Apache-2.0 license

**Choice**: Single Apache-2.0 license.

**Alternatives considered**:
- MIT: Simpler but no patent grant.
- MIT OR Apache-2.0 (Rust ecosystem convention): Dual license gives users choice.

**Rationale**: Apache-2.0 provides explicit patent grant and retaliation clause. Single license is simpler than dual licensing. Sufficient for Phase 2b crates.io publishing (which requires a license field).

### D8: Tarball contents and checksums

**Choice**: Each tarball contains the binary + LICENSE file. A separate `checksums.txt` release asset contains SHA256 hashes of all 4 tarballs.

**Rationale**: Follows established Rust CLI conventions (ripgrep, bat). LICENSE inclusion is legally proper. SHA256 checksums are cheap to generate and enable download verification.

### D9: No pre-tagging for initial release

**Choice**: Let release-plz handle the initial release without creating a `v0.1.0` tag first.

**Alternatives considered**:
- Tag `v0.1.0` manually before enabling release-plz to anchor the baseline and limit changelog scope.

**Rationale**: With `git_only = true` and no existing tags, release-plz falls back to the version in `Cargo.toml` (0.1.0) and treats the package as an initial release. The first CHANGELOG will cover the full 133-commit project history. This is intentional — the first release documents the complete development record.

### D10: Single-package release from workspace

**Choice**: Only `atlassy-cli` produces releases. The 4 internal crates (`atlassy-pipeline`, `atlassy-adf`, `atlassy-confluence`, `atlassy-contracts`) are excluded via `release = false` at workspace level.

**Rationale**: Internal crates are implementation details, not independently distributable. All 5 crates share the workspace version and move in lockstep. The `changelog_include` setting ensures commits to any crate appear in the CLI's changelog. release-plz modifies the workspace version in root `Cargo.toml`; all crates inherit via `version.workspace = true`.

## Risks / Trade-offs

- [ARM64 Linux runner (`ubuntu-24.04-arm`) is a partner image] → Preinstalled software may differ from `ubuntu-latest`. Mitigated by installing Rust toolchain explicitly via `dtolnay/rust-toolchain@stable`.
- [First release changelog includes full project history (133 commits)] → May be long but provides complete record. Subsequent releases will have focused changelogs.
- [GitHub repo setting must be changed manually] → Document in prerequisites. If forgotten, release-plz-pr job will fail with a permissions error — visible and diagnosable.
- [`release_always = false` adds one extra merge step per release] → Acceptable trade-off for controlled release timing. The release PR also serves as a review checkpoint.
- [No crates.io publishing] → `cargo install atlassy-cli` won't work until Phase 2b. Mitigated by documenting `curl | tar` install in README.
