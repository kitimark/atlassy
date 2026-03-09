## Purpose

Define build-time provenance stamping requirements for decision-grade outputs and replay-safe sign-off.

## Requirements

### Requirement: Decision-grade outputs include provenance stamp
The system MUST include a provenance stamp on decision-grade artifacts with `git_commit_sha`, `git_dirty`, and `pipeline_version`. The values for `git_commit_sha` and `git_dirty` SHALL be compile-time constants embedded in the binary.

#### Scenario: Emit provenance on run summary
- **WHEN** a run summary is persisted
- **THEN** the summary includes `git_commit_sha`, `git_dirty`, and `pipeline_version`

#### Scenario: Emit provenance on batch and readiness outputs
- **WHEN** batch report and readiness artifacts are generated
- **THEN** each artifact includes the same provenance stamp fields

### Requirement: Provenance values are embedded at compile time
The system SHALL embed `git_commit_sha` and `git_dirty` into the CLI binary at compile time via `build.rs`. The CLI MUST NOT resolve these values by spawning git subprocesses at runtime.

#### Scenario: Embedded SHA is available without git at runtime
- **WHEN** the CLI binary is executed in an environment without git installed
- **THEN** `collect_provenance()` succeeds and returns a valid `ProvenanceStamp` with `git_commit_sha` and `git_dirty` populated

#### Scenario: Embedded SHA matches the build commit
- **WHEN** the CLI binary is compiled at a specific git commit
- **THEN** the `git_commit_sha` in every `ProvenanceStamp` produced by that binary equals the 40-character SHA of that commit

### Requirement: Build fails without valid git context
The build process SHALL fail if `git rev-parse HEAD` is unavailable or returns a malformed SHA during compilation.

#### Scenario: Build without git repository fails
- **WHEN** `cargo build` runs outside a git repository
- **THEN** the build fails with a clear error message

#### Scenario: Malformed SHA fails the build
- **WHEN** `git rev-parse HEAD` returns a value that is not a 40-character hex string during compilation
- **THEN** the build fails with a clear error message

### Requirement: Missing provenance blocks decision claims
The system SHALL reject KPI or readiness decision outputs when required provenance fields are missing or malformed.

#### Scenario: Missing commit SHA prevents decision output
- **WHEN** `git_commit_sha` is absent or not a 40-character SHA
- **THEN** readiness output generation fails with deterministic diagnostics

### Requirement: Provenance consistency across related artifacts
Artifacts produced from the same execution context MUST use a consistent provenance stamp.

#### Scenario: Inconsistent provenance is rejected
- **WHEN** run, batch, and readiness outputs contain conflicting provenance values for the same execution context
- **THEN** replay verification fails and sign-off remains blocked
