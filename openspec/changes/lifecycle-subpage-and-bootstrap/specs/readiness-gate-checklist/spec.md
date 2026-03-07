## MODIFIED Requirements

### Requirement: Deterministic readiness gate evaluation
The readiness workflow MUST evaluate v1 Gate 1 through Gate 7 in deterministic order and emit a normalized result record for each gate.

#### Scenario: Evaluate all readiness gates in order
- **WHEN** batch evidence is available for a release-readiness run
- **THEN** the system evaluates Gate 1, Gate 2, Gate 3, Gate 4, Gate 5, Gate 6, and Gate 7 in fixed order
- **AND** each gate result includes gate name, target, pass/fail state, and evidence references

## ADDED Requirements

### Requirement: Gate 7 validates lifecycle enablement evidence
Gate 7 (Lifecycle Enablement Validation) SHALL pass only when batch evidence includes deterministic outcomes for all four lifecycle matrix paths: blank subpage creation, bootstrap-required failure, bootstrap success, and bootstrap-on-non-empty failure.

#### Scenario: Gate 7 passes with complete lifecycle evidence
- **WHEN** batch evidence includes at least one run demonstrating each lifecycle matrix outcome
- **THEN** Gate 7 passes
- **AND** the gate result references lifecycle evidence artifacts

#### Scenario: Gate 7 fails with missing lifecycle evidence
- **WHEN** batch evidence is missing any lifecycle matrix outcome
- **THEN** Gate 7 fails
- **AND** the blocking reason identifies which lifecycle evidence is missing

#### Scenario: Gate 7 failure blocks readiness sign-off
- **WHEN** Gate 7 fails
- **THEN** readiness sign-off is marked as blocked
- **AND** the decision packet recommendation is `iterate` or lower
