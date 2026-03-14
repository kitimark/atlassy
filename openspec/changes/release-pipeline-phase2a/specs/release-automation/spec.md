## ADDED Requirements

### Requirement: release-plz configuration file
The repository SHALL contain a `release-plz.toml` at the workspace root that configures release-plz for GitHub-only releases. The configuration SHALL set `publish = false`, `git_only = true`, `release_always = false`, `semver_check = false`, and `features_always_increment_minor = true` at the workspace level. Only the `atlassy-cli` package SHALL have `release = true`. The `changelog_include` setting SHALL list all 4 internal crates so their commits appear in the CLI changelog.

#### Scenario: Configuration file exists with correct settings
- **WHEN** release-plz reads `release-plz.toml`
- **THEN** it operates in git-only mode without crates.io interaction, only processes `atlassy-cli` for releases, and includes commits from all workspace crates in the changelog

#### Scenario: Internal crates do not produce releases
- **WHEN** release-plz evaluates the workspace
- **THEN** `atlassy-pipeline`, `atlassy-adf`, `atlassy-confluence`, and `atlassy-contracts` are skipped for release processing because `release = false` at workspace level

### Requirement: Release PR workflow
The repository SHALL contain `.github/workflows/release-plz.yml` triggered on push to `main`. The workflow SHALL run two jobs: `release-plz release-pr` to create or update a release PR, and `release-plz release` to create a git tag and GitHub Release when a release PR is merged. Both jobs SHALL pass `--git-token` to release-plz.

#### Scenario: Feature commit pushed to main
- **WHEN** a commit with type `feat:` is pushed to `main`
- **THEN** release-plz creates or updates a release PR that bumps the workspace version (minor increment) in root `Cargo.toml` and generates or updates `CHANGELOG.md`

#### Scenario: Fix commit pushed to main
- **WHEN** a commit with type `fix:` is pushed to `main`
- **THEN** release-plz creates or updates a release PR that bumps the workspace version (patch increment) in root `Cargo.toml` and generates or updates `CHANGELOG.md`

#### Scenario: Release PR is merged
- **WHEN** a release PR (branch prefix `release-plz-`) is merged to `main`
- **THEN** release-plz creates a git tag (e.g., `v0.2.0`) and a GitHub Release with the changelog as the body

#### Scenario: Non-release PR is merged
- **WHEN** a regular (non-release) PR is merged to `main`
- **THEN** the release job is a no-op because `release_always = false`

### Requirement: Release workflow uses GitHub App token
The release workflow SHALL generate a short-lived token using `actions/create-github-app-token` with `APP_ID` and `APP_PRIVATE_KEY` secrets and SHALL use that token for both release-plz jobs.

#### Scenario: App token available
- **WHEN** `APP_ID` and `APP_PRIVATE_KEY` secrets are configured
- **THEN** release-plz uses the generated App token for `release-pr` and `release`, and release-created tags can trigger downstream workflows

#### Scenario: App token secrets missing
- **WHEN** either `APP_ID` or `APP_PRIVATE_KEY` is not configured
- **THEN** the workflow fails before running release-plz with a clear error

#### Scenario: Release PR goes through CI
- **WHEN** release-plz creates a release PR
- **THEN** the existing `test` job (fmt-check, clippy, tests) runs on the PR and MUST pass before merge

### Requirement: Versioning convention
The release pipeline SHALL use `features_always_increment_minor = true` so that `feat:` commits bump the minor version (e.g., 0.1.0 → 0.2.0) and `fix:` commits bump the patch version (e.g., 0.1.0 → 0.1.1). Breaking changes (`feat!:` or `BREAKING CHANGE:`) SHALL bump the minor version in pre-1.0 releases.

#### Scenario: feat commit bumps minor
- **WHEN** the unreleased commits include at least one `feat:` commit
- **THEN** the release PR proposes a minor version bump

#### Scenario: fix-only commits bump patch
- **WHEN** all unreleased commits are `fix:` type (no `feat:`)
- **THEN** the release PR proposes a patch version bump

### Requirement: Workflow uses checkout v4 with full history
Both release workflows SHALL use `actions/checkout@v4` with `fetch-depth: 0` and `persist-credentials: false`. This is consistent with the Phase 1 CI workflow checkout version.

#### Scenario: Checkout configuration
- **WHEN** release-plz.yml checks out the repository
- **THEN** it fetches full git history (for commit analysis) and does not persist credentials (required by release-plz)

### Requirement: Workflow permissions
The `release-plz.yml` workflow SHALL declare `contents: write` and `pull-requests: write` permissions.

#### Scenario: Permissions are sufficient
- **WHEN** release-plz attempts to create a release PR and a GitHub Release
- **THEN** it succeeds because the workflow has write access to contents and pull requests

### Requirement: Initial release bootstrapping
The release pipeline SHALL handle the initial release without any pre-existing git tags. release-plz SHALL fall back to the version in `Cargo.toml` (0.1.0) and treat the package as an initial release, producing a changelog covering the full commit history.

#### Scenario: First run with no tags
- **WHEN** release-plz runs for the first time with no git tags in the repository
- **THEN** it creates a release PR bumping from 0.1.0 to 0.2.0 (assuming `feat:` commits exist) with a changelog covering all commits since the initial commit

### Requirement: Apache-2.0 license file
The repository SHALL contain a `LICENSE` file with the Apache-2.0 license text.

#### Scenario: LICENSE file exists
- **WHEN** a user or automated tool checks the repository for license information
- **THEN** a `LICENSE` file with Apache-2.0 text is present at the repository root

### Requirement: GitHub Actions PR creation permission
The repository SHALL have `can_approve_pull_request_reviews` set to `true` in GitHub Actions workflow permissions so that release-plz can create release PRs. This SHALL be enabled via the GitHub API: `gh api repos/kitimark/atlassy/actions/permissions/workflow -X PUT -f default_workflow_permissions="read" -F can_approve_pull_request_reviews=true`.

#### Scenario: release-plz can create PRs
- **WHEN** release-plz attempts to create a release PR
- **THEN** it succeeds because GitHub Actions has permission to create pull requests

#### Scenario: Permission not enabled
- **WHEN** `can_approve_pull_request_reviews` is `false` and release-plz attempts to create a PR
- **THEN** the `release-plz-pr` job fails with a permissions error

### Requirement: Local release build target
The Makefile SHALL include a `build-release` target that runs `cargo build -p atlassy-cli --release` to produce a release-optimized binary.

#### Scenario: make build-release
- **WHEN** an operator runs `make build-release`
- **THEN** a release binary is produced at `target/release/atlassy-cli`

### Requirement: Path dependencies are packageable for release-plz
Workspace path dependencies in releasable crates SHALL include explicit version requirements so `cargo package` checks performed by release-plz succeed against historical baselines.

#### Scenario: release-plz validates package metadata
- **WHEN** release-plz computes the next release and runs `cargo package`
- **THEN** packaging succeeds because internal path dependencies used by releasable crates include explicit version requirements
