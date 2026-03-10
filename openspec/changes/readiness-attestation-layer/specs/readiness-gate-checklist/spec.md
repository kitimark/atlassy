## MODIFIED Requirements

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
