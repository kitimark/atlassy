## ADDED Requirements

### Requirement: CI workflow triggers on push and pull request
The CI workflow SHALL run on every push to the `main` branch and on every pull request targeting the `main` branch.

#### Scenario: Push to main triggers CI
- **WHEN** a commit is pushed to the `main` branch
- **THEN** the CI workflow runs the full test job

#### Scenario: Pull request triggers CI
- **WHEN** a pull request targeting `main` is opened or updated
- **THEN** the CI workflow runs the full test job

### Requirement: CI checks formatting
The CI workflow SHALL reject code that does not pass `cargo fmt --all -- --check`.

#### Scenario: Unformatted code fails CI
- **WHEN** the workspace contains unformatted Rust code
- **THEN** the format check step fails and the workflow reports failure

#### Scenario: Formatted code passes CI
- **WHEN** all Rust code in the workspace passes `cargo fmt` check
- **THEN** the format check step succeeds

### Requirement: CI checks lint warnings
The CI workflow SHALL reject code that produces clippy warnings, using `cargo clippy --workspace --all-targets -- -D warnings`.

#### Scenario: Clippy warning fails CI
- **WHEN** the workspace contains code producing clippy warnings
- **THEN** the lint step fails and the workflow reports failure

#### Scenario: Clean clippy passes CI
- **WHEN** the workspace produces zero clippy warnings
- **THEN** the lint step succeeds

### Requirement: CI runs full test suite
The CI workflow SHALL run `cargo test --workspace` and reject code with any test failure.

#### Scenario: Test failure fails CI
- **WHEN** any test in the workspace fails
- **THEN** the test step fails and the workflow reports failure

#### Scenario: All tests pass
- **WHEN** all workspace tests pass
- **THEN** the test step succeeds

### Requirement: CI steps execute in fail-fast order
The CI workflow SHALL execute steps in this order: commit message check, format check, clippy lint, test. Each step SHALL only run if all preceding steps succeeded.

#### Scenario: Format failure skips lint and test
- **WHEN** the format check fails
- **THEN** clippy lint and test steps do not execute

### Requirement: CI uses Cargo dependency caching
The CI workflow SHALL cache the `target/` directory and cargo registry keyed by `Cargo.lock` hash to reduce build times on repeat runs.

#### Scenario: Cache hit reduces build time
- **WHEN** `Cargo.lock` has not changed since the last CI run
- **THEN** the cached `target/` directory and registry are restored before compilation

### Requirement: Branch protection gates merges on CI
The repository SHALL require the CI test job to pass before merging to `main`. The branch MUST be up-to-date with `main` before merging.

#### Scenario: Failed CI blocks merge
- **WHEN** the CI test job has not passed for a PR
- **THEN** merging to `main` is blocked

#### Scenario: Passing CI allows merge
- **WHEN** the CI test job passes and the branch is up-to-date
- **THEN** merging to `main` is allowed

### Requirement: Repository enforces squash-only merges
The repository SHALL allow only squash merges. Merge commits and rebase merges SHALL be disabled. The squash commit message SHALL use the PR title.

#### Scenario: Squash merge uses PR title
- **WHEN** a PR is merged via squash
- **THEN** the resulting commit message on `main` is the PR title

#### Scenario: Non-squash merge methods are unavailable
- **WHEN** a contributor attempts to merge a PR
- **THEN** only the squash merge option is available
