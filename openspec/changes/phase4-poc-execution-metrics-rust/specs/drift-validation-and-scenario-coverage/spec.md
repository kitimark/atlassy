## ADDED Requirements

### Requirement: Live-vs-stub drift validation gate
PoC workflow MUST compare scheduled live smoke probe outcomes to stub scenario expectations for key behaviors and MUST block sign-off when unresolved material drift is detected.

#### Scenario: Material behavior drift detected
- **WHEN** live smoke output disagrees with stub expectations on scoped fetch, publish conflict handling, or required error payload behavior
- **THEN** drift status is marked unresolved
- **AND** PoC sign-off is blocked until drift resolution is documented

### Requirement: Required scenario coverage validation
PoC execution MUST validate coverage of required v1 scenario IDs for positive and negative safety paths before final reporting.

#### Scenario: Missing required scenario coverage
- **WHEN** one or more required scenario IDs are absent from the batch evidence set
- **THEN** batch status is marked incomplete
- **AND** final recommendation output is gated pending coverage completion

### Requirement: Safety-gate violations are surfaced as hard blockers
Any locked-node mutation, out-of-scope mutation, or table-shape policy violation observed in batch runs MUST be surfaced as hard blockers in final reporting.

#### Scenario: Safety violation observed
- **WHEN** run diagnostics include a safety-gate failure for locked-node mutation, out-of-scope mutation, or table-shape violation
- **THEN** final report marks the batch as safety-failed
- **AND** recommendation is constrained to non-go outcomes until resolved
