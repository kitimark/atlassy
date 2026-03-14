## Why

Atlassy has no release infrastructure. There is no way to produce versioned binaries, generate changelogs, create git tags, or distribute builds to users. CI (Phase 1) covers quality gates only. Users must clone the repo and build from source. Adding a release pipeline enables reproducible binary distribution, version-tracked releases, and a clear install path.

## What Changes

- Add `release-plz` automation: release PRs with version bump + CHANGELOG on push to main, git tag + GitHub Release on release PR merge (controlled via `release_always = false`).
- Add cross-platform binary build workflow triggered by tag push — 4 native targets (x86_64/aarch64 Linux, x86_64/aarch64 macOS) using architecture-matched GitHub runners. No cross-compilation tooling required.
- Add `release-plz.toml` configuration (`git_only = true`, `features_always_increment_minor = true`, single-package release for `atlassy-cli` with changelog including all internal crate commits).
- Add `LICENSE` file (Apache-2.0).
- Add `build-release` Makefile target (`cargo build -p atlassy-cli --release`).
- Add platform-specific install instructions to `README.md`.

## Capabilities

### New Capabilities

- `release-automation`: release-plz configuration, release PR workflow, tag and GitHub Release creation, versioning convention (`feat` bumps minor in 0.x), controlled release cadence.
- `binary-distribution`: cross-platform build matrix (4 native targets), tarball packaging (binary + LICENSE), SHA256 checksum generation, GitHub Release asset upload via `gh release upload`.

### Modified Capabilities

None. This is pure CI/CD infrastructure — no application behavior changes.

## Impact

- New files: `release-plz.toml`, `.github/workflows/release-plz.yml`, `.github/workflows/release-build.yml`, `LICENSE`.
- Modified files: `Makefile` (add `build-release` target), `README.md` (add install section).
- CI: existing `test` job is unaffected and continues to gate all PRs including release PRs.
- Manual prerequisite: enable "Allow GitHub Actions to create and approve pull requests" in GitHub repo settings (Settings → Actions → General).
- Versioning: first release PR will bump from 0.1.0 to 0.2.0 with full project history in CHANGELOG.
