## Purpose

Define deterministic v1 release-readiness gate evaluation so sign-off outcomes are auditable, role-attributed, and consistently blocked when mandatory conditions fail.

## Requirements

### Requirement: Deterministic readiness gate evaluation
The readiness workflow MUST evaluate v1 Gate 1 through Gate 7 in deterministic order and emit a normalized result record for each gate.

#### Scenario: Evaluate all readiness gates in order
- **WHEN** batch evidence is available for a release-readiness run
- **THEN** the system evaluates Gate 1, Gate 2, Gate 3, Gate 4, Gate 5, Gate 6, and Gate 7 in fixed order
- **AND** each gate result includes gate name, target, pass/fail state, and evidence references

### Requirement: Gate 7 validates lifecycle enablement evidence
Gate 7 (Lifecycle Enablement Validation) SHALL pass when readiness evidence includes deterministic outcomes for all four lifecycle matrix paths: blank subpage creation, bootstrap-required failure, bootstrap success, and bootstrap-on-non-empty failure. Evidence MAY come from batch run summaries, from a `lifecycle_validation` attestation entry, or from a combination of both sources. Each lifecycle condition is evaluated independently using `||` composition across both sources.

#### Scenario: Gate 7 passes with complete lifecycle evidence from batch summaries
- **WHEN** batch run summaries include at least one run demonstrating each lifecycle matrix outcome
- **AND** no attestation file is present
- **THEN** Gate 7 passes
- **AND** the gate result references lifecycle evidence artifacts

#### Scenario: Gate 7 passes with complete lifecycle evidence from attestation
- **WHEN** a `lifecycle_validation` attestation is present with all four claims set to `true`
- **AND** batch run summaries do not include lifecycle runs
- **THEN** Gate 7 passes

#### Scenario: Gate 7 passes with mixed evidence from both sources
- **WHEN** some lifecycle conditions are satisfied by batch run summaries and remaining conditions are satisfied by a `lifecycle_validation` attestation
- **THEN** Gate 7 passes

#### Scenario: Gate 7 fails with missing lifecycle evidence
- **WHEN** neither batch run summaries nor attestation entries provide complete lifecycle matrix coverage
- **THEN** Gate 7 fails
- **AND** the blocking reason identifies which lifecycle evidence is missing

#### Scenario: Gate 7 failure blocks readiness sign-off
- **WHEN** Gate 7 fails
- **THEN** readiness sign-off is marked as blocked
- **AND** the decision packet recommendation is `iterate` or lower

#### Scenario: Malformed attestation claims do not cause false pass
- **WHEN** a `lifecycle_validation` attestation is present but its claims do not deserialize into the expected structure
- **AND** batch run summaries do not include lifecycle runs
- **THEN** Gate 7 fails as if no attestation were present

### Requirement: Mandatory gate failures block readiness sign-off
The readiness workflow SHALL mark sign-off as blocked when any mandatory gate fails or required evidence is missing.

#### Scenario: Block sign-off on failed gate
- **WHEN** one or more readiness gates fail
- **THEN** sign-off status is marked blocked
- **AND** the output lists each failed gate and its blocking reason

### Requirement: Checklist outputs are auditable and role-attributed
The readiness checklist output MUST include explicit owner-role attribution, generation timestamp, and immutable links to source artifacts for audit review.

#### Scenario: Produce audit-ready checklist
- **WHEN** readiness checklist generation completes
- **THEN** output includes owner roles, generation timestamp, and source artifact paths
- **AND** reviewers can trace every gate decision to persisted evidence
