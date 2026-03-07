# CI and Automation

## Objective

Establish automated quality gates that validate code correctness, style, and safety on every push and pull request, preventing regressions before merge.

## Current Baseline

- All testing has been manual: `cargo fmt`, `cargo clippy`, `cargo test` run locally.
- No automated CI pipeline exists.
- 82 tests across 7 binaries (5 crates) run reliably in under 60 seconds.
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
| Toolchain | `dtolnay/rust-toolchain@stable` | Install Rust (reads `rust-toolchain.toml`) |
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

## Phase 2: Build and Artifact Validation (Deferred)

### Scope

Add build artifact steps and optional artifact upload for release validation.

### Candidate Work

- Add `cargo build --release --workspace` step.
- Upload binary artifacts for tagged releases.
- Add version tagging and changelog automation.

### Signals to Start

- Project approaches v1 release readiness (`go` recommendation).
- Need for distributable binaries beyond local development.

## Phase 3: Extended Automation (Deferred)

### Scope

Additional CI capabilities beyond basic test/build.

### Candidate Work

- Matrix builds (multiple Rust versions, multiple OS targets).
- Code coverage reporting (e.g., `cargo-llvm-cov` with Codecov or similar).
- Live Confluence smoke tests in CI (requires GitHub Actions secrets for sandbox credentials).
- Scheduled nightly runs for drift detection.
- Dependency audit (`cargo-deny` or `cargo-audit`).

### Signals to Start

- Test job is stable and passing consistently.
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
