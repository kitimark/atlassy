## Purpose

Define deterministic v1 pipeline orchestration behavior, including state order, hard-error halting, and contract-valid state envelopes.

## Requirements

### Requirement: Deterministic v1 state execution order
The pipeline orchestrator SHALL execute v1 states in this exact order: `fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> merge_candidates -> patch -> verify -> publish`, and MUST ensure the `patch` state output candidate payload is the payload evaluated by `verify` and attempted by `publish`.

#### Scenario: All states succeed in order
- **WHEN** a run starts with valid input and no state returns a hard error
- **THEN** the orchestrator executes each state exactly once in the defined order and marks the run successful after `publish`

#### Scenario: State order mismatch is prevented
- **WHEN** a state transition is attempted out of the defined sequence
- **THEN** the orchestrator MUST fail the run with a deterministic transition error and MUST NOT execute downstream states

#### Scenario: Patch output is propagated to verify and publish
- **WHEN** `patch` produces updated `candidate_page_adf`
- **THEN** `verify` evaluates the updated candidate payload
- **AND** `publish` receives the same verified payload

### Requirement: Fail-fast behavior on hard errors
The orchestrator MUST halt on the first hard error returned by any state and SHALL emit a run result that includes the originating state and a typed `ErrorCode` variant. Error codes MUST be `ErrorCode` enum values, not strings. Deterministic mapping from upstream error types (`AdfError`, `ConfluenceError`) to `ErrorCode` variants MUST be compiler-verified via exhaustive match.

#### Scenario: Verify hard error blocks publish
- **WHEN** `verify` returns a hard error such as `ErrorCode::SchemaInvalid`
- **THEN** the orchestrator stops immediately, does not call `publish`, and records the failure in the run summary

#### Scenario: Live backend hard error is mapped and halts run
- **WHEN** live fetch or publish returns a hard backend failure
- **THEN** the orchestrator records a typed `ErrorCode` variant mapped from the `ConfluenceError` variant and halts before executing downstream states

#### Scenario: Error code is serialized to original string in run summary
- **WHEN** a hard error with an `ErrorCode` variant is recorded in `RunSummary.error_codes`
- **THEN** the string value MUST match the original `ERR_*` constant (e.g., `ErrorCode::SchemaInvalid` serializes as `"ERR_SCHEMA_INVALID"`)

### Requirement: Contract-valid state envelopes
Each state input and output envelope SHALL include required metadata fields (`request_id`, `page_id`, `state`, `timestamp`) and MUST be validated before transition to the next state.

#### Scenario: Missing required envelope field
- **WHEN** a state output omits a required metadata field
- **THEN** the orchestrator fails the run with a contract validation error and blocks subsequent states
