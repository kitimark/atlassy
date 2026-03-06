## Purpose

Define deterministic paired PoC batch execution behavior for baseline and optimized flows, including manifest validation and retry-policy invariants.

## Requirements

### Requirement: Deterministic paired run matrix execution
The PoC runner SHALL execute a deterministic matrix of runs keyed by `(page_id, pattern, flow, edit_intent_hash)` and MUST enforce paired baseline and optimized runs for each matrix key.

#### Scenario: Execute complete paired matrix
- **WHEN** a run manifest includes Pattern A/B/C entries for a configured page set with both `baseline` and `optimized` flows
- **THEN** the runner executes all manifest entries in deterministic order
- **AND** each `(page_id, pattern, edit_intent_hash)` key has one baseline run and one optimized run

### Requirement: Manifest identity and completeness validation
The runner MUST reject manifests that contain duplicate `run_id` values, missing required identity fields, or unmatched baseline/optimized pairings.

#### Scenario: Reject unmatched pair
- **WHEN** a manifest contains an optimized run without a matching baseline run for the same matrix key
- **THEN** execution fails before batch start
- **AND** diagnostics identify the unmatched matrix key

### Requirement: Retry policy invariants are enforced during PoC execution
PoC execution MUST enforce the v1 one-scoped-retry maximum and treat retry-policy breaches as batch-level failure conditions.

#### Scenario: Retry limit exceeded
- **WHEN** any run attempts more than one scoped publish retry
- **THEN** the run is marked failed
- **AND** the batch status records a retry-policy violation
