## 1. License and Makefile

- [x] 1.1 Create `LICENSE` file with Apache-2.0 text at repository root
- [x] 1.2 Add `build-release` target to `Makefile` (`cargo build -p atlassy-cli --release`) and add to `.PHONY` list
- [x] 1.3 Verify `make build-release` produces binary at `target/release/atlassy-cli`

## 2. release-plz Configuration

- [x] 2.1 Create `release-plz.toml` at repository root with workspace settings: `publish = false`, `git_only = true`, `release_always = false`, `semver_check = false`, `features_always_increment_minor = true`, `release = false`
- [x] 2.2 Add `[[package]]` section for `atlassy-cli` with `release = true` and `changelog_include` listing all 4 internal crates

## 3. Release PR Workflow

- [x] 3.1 Create `.github/workflows/release-plz.yml` with trigger on push to `main`
- [x] 3.2 Add `release-plz-pr` job: checkout with `fetch-depth: 0` and `persist-credentials: false`, install release-plz, run `release-plz release-pr`
- [x] 3.3 Add `release-plz-release` job: checkout with `fetch-depth: 0` and `persist-credentials: false`, install release-plz, run `release-plz release`
- [x] 3.4 Set workflow permissions: `contents: write`, `pull-requests: write`

## 4. Binary Build Workflow

- [x] 4.1 Create `.github/workflows/release-build.yml` with trigger on tag push matching `v*`
- [x] 4.2 Define build matrix with 4 entries: `x86_64-unknown-linux-gnu` / `ubuntu-latest`, `aarch64-unknown-linux-gnu` / `ubuntu-24.04-arm`, `x86_64-apple-darwin` / `macos-15-intel`, `aarch64-apple-darwin` / `macos-latest`
- [x] 4.3 Add build job steps: checkout, setup Rust toolchain (`dtolnay/rust-toolchain@stable`), `cargo build -p atlassy-cli --release`
- [x] 4.4 Add packaging step: create tarball `atlassy-cli-{tag}-{target}.tar.gz` containing binary and LICENSE
- [x] 4.5 Add upload step: `gh release upload` to attach tarball to existing GitHub Release
- [x] 4.6 Add checksums job: download all 4 tarballs, generate `checksums.txt` with SHA256 hashes, upload to GitHub Release

## 5. README Install Instructions

- [x] 5.1 Add Installation section to `README.md` with `curl | tar` commands for all 4 platforms (macOS Apple Silicon, macOS Intel, Linux x86_64, Linux ARM64)

## 6. Commit and Verify

- [x] 6.1 Commit all new and modified files with message `ci: add release pipeline with release-plz and cross-platform builds`
- [x] 6.2 Enable GitHub Actions PR creation permission: `gh api repos/kitimark/atlassy/actions/permissions/workflow -X PUT -f default_workflow_permissions="read" -F can_approve_pull_request_reviews=true`
- [x] 6.3 Add explicit versions for internal path dependencies in releasable crates (`crates/atlassy-cli/Cargo.toml`, `crates/atlassy-pipeline/Cargo.toml`) so release-plz `cargo package` checks pass
- [x] 6.4 Switch `.github/workflows/release-plz.yml` to GitHub App token flow (`actions/create-github-app-token`) using `APP_ID` and `APP_PRIVATE_KEY`
- [x] 6.5 Add fail-fast preflight checks in `release-plz.yml` for missing `APP_ID` / `APP_PRIVATE_KEY` secrets before running release-plz
- [x] 6.6 Fix `.github/workflows/release-build.yml` checksums job to use explicit repository context for `gh release download` and `gh release upload` (no `.git` dependency)
- [ ] 6.7 Re-run release flow and verify release-plz creates/updates release PR and merged release triggers `release-build.yml`
- [x] 6.8 Verify release has 5 assets (4 tarballs + `checksums.txt`) and smoke-test checksum + extracted binary via `gh` CLI commands in `design.md`
