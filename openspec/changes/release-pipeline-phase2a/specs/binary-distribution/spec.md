## ADDED Requirements

### Requirement: Cross-platform build workflow
The repository SHALL contain `.github/workflows/release-build.yml` triggered on tag push matching `v*`. The workflow SHALL build release binaries for 4 targets using a matrix strategy with architecture-matched native runners.

#### Scenario: Tag push triggers build
- **WHEN** a git tag matching `v*` (e.g., `v0.2.0`) is pushed
- **THEN** the release-build workflow starts and runs 4 parallel matrix jobs

#### Scenario: Build matrix targets
- **WHEN** the build matrix executes
- **THEN** it builds `atlassy-cli` in release mode for `x86_64-unknown-linux-gnu` on `ubuntu-latest`, `aarch64-unknown-linux-gnu` on `ubuntu-24.04-arm`, `x86_64-apple-darwin` on `macos-15-intel`, and `aarch64-apple-darwin` on `macos-latest`

#### Scenario: All builds use native compilation
- **WHEN** each matrix job runs
- **THEN** it uses `cargo build -p atlassy-cli --release` directly on the runner without cross-compilation tools

### Requirement: Tarball packaging
Each build job SHALL package the release binary and LICENSE file into a gzipped tarball named `atlassy-cli-{tag}-{target}.tar.gz`.

#### Scenario: Tarball naming
- **WHEN** the build for `x86_64-unknown-linux-gnu` completes with tag `v0.2.0`
- **THEN** it produces a tarball named `atlassy-cli-v0.2.0-x86_64-unknown-linux-gnu.tar.gz`

#### Scenario: Tarball contents
- **WHEN** a user extracts a release tarball
- **THEN** it contains the `atlassy-cli` binary and the `LICENSE` file

### Requirement: SHA256 checksum generation
The workflow SHALL generate a `checksums.txt` file containing SHA256 hashes of all 4 tarballs, uploaded as a separate release asset.

#### Scenario: Checksums file content
- **WHEN** all 4 build jobs complete
- **THEN** a `checksums.txt` file is generated with SHA256 hashes for all 4 tarballs

#### Scenario: Checksum verification
- **WHEN** a user downloads a tarball and `checksums.txt`
- **THEN** they can verify the tarball integrity by running `shasum -a 256 -c checksums.txt`

### Requirement: GitHub Release asset upload
Each build job SHALL upload its tarball to the existing GitHub Release (created by release-plz) using `gh release upload`. The checksums job SHALL upload `checksums.txt` to the same release.

#### Scenario: Assets attached to release
- **WHEN** all build and checksum jobs complete for tag `v0.2.0`
- **THEN** the GitHub Release for `v0.2.0` has 5 assets: 4 tarballs and 1 `checksums.txt`

#### Scenario: Upload uses gh CLI
- **WHEN** a build job uploads its tarball
- **THEN** it uses `gh release upload` with `GITHUB_TOKEN` authentication, not a third-party action

### Requirement: Rust toolchain setup in build jobs
Each build job SHALL install the Rust stable toolchain using `dtolnay/rust-toolchain@stable`, consistent with the Phase 1 CI workflow.

#### Scenario: Toolchain installation
- **WHEN** a build matrix job starts
- **THEN** it installs the stable Rust toolchain before building

### Requirement: README install instructions
`README.md` SHALL include platform-specific install instructions for all 4 targets, showing how to download and extract the binary from GitHub Releases.

#### Scenario: Install instructions present
- **WHEN** a user reads the README
- **THEN** they find `curl | tar` install commands for macOS (Apple Silicon), macOS (Intel), Linux (x86_64), and Linux (ARM64)

#### Scenario: Install instructions reference latest release
- **WHEN** install instructions reference a download URL
- **THEN** the URL pattern uses the GitHub Releases download path for the repository
