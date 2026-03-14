## Why

Atlassy has no release infrastructure. There is no way to produce versioned binaries, generate changelogs, create git tags, or distribute builds to users. CI (Phase 1) covers quality gates only. Users must clone the repo and build from source. Adding a release pipeline enables reproducible binary distribution, version-tracked releases, and a clear install path.

## What Changes

- Harden `release-plz` automation to use a GitHub App token (`actions/create-github-app-token`) for release PR and tag/release creation so downstream workflows can trigger from release events.
- Keep controlled release cadence via `release_always = false` and release PR merges.
- Add cross-platform binary build workflow triggered by tag push and manual dispatch fallback for replay/recovery — 4 native targets (x86_64/aarch64 Linux, x86_64/aarch64 macOS) using architecture-matched GitHub runners.
- Fix release checksum generation/upload reliability by making `gh release download` repository-context safe in the checksums job.
- Add `release-plz.toml` configuration (`git_only = true`, `features_always_increment_minor = true`, single-package release for `atlassy-cli` with changelog including all internal crate commits).
- Add `LICENSE` file (Apache-2.0).
- Add `build-release` Makefile target (`cargo build -p atlassy-cli --release`).
- Add platform-specific install instructions to `README.md`.
- Expand post-merge verification and troubleshooting steps to cover token setup, release workflow chaining, and release asset completeness.

## Capabilities

### New Capabilities

- `release-automation`: release-plz configuration, release PR workflow, tag and GitHub Release creation, versioning convention (`feat` bumps minor in 0.x), controlled release cadence.
- `binary-distribution`: cross-platform build matrix (4 native targets), tarball packaging (binary + LICENSE), SHA256 checksum generation, GitHub Release asset upload via `gh release upload`.

### Modified Capabilities

None. This is pure CI/CD infrastructure — no application behavior changes.

## Impact

- New files: `release-plz.toml`, `.github/workflows/release-plz.yml`, `.github/workflows/release-build.yml`, `LICENSE`.
- Modified files: `Makefile` (add `build-release` target), `README.md` (add install section).
- GitHub Secrets: requires `APP_ID` and `APP_PRIVATE_KEY` for the GitHub App token used by release-plz.
- CI: existing `test` job is unaffected and continues to gate all PRs including release PRs.
- Manual prerequisite: enable "Allow GitHub Actions to create and approve pull requests" in GitHub repo settings (Settings → Actions → General).
- Versioning: first release PR will bump from 0.1.0 to 0.2.0 with full project history in CHANGELOG.
- Recovery path: `release-build` can be manually re-run for an existing release tag via `workflow_dispatch`.
