## ADDED Requirements

### Requirement: Decision-grade outputs include provenance stamp
The system MUST include a provenance stamp on decision-grade artifacts with `git_commit_sha`, `git_dirty`, and `pipeline_version`.

#### Scenario: Emit provenance on run summary
- **WHEN** a run summary is persisted
- **THEN** the summary includes `git_commit_sha`, `git_dirty`, and `pipeline_version`

#### Scenario: Emit provenance on batch and readiness outputs
- **WHEN** batch report and readiness artifacts are generated
- **THEN** each artifact includes the same provenance stamp fields

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
