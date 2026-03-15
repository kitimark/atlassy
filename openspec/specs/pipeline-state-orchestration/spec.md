## Purpose

Define deterministic v1 pipeline orchestration behavior, including state order, hard-error halting, and contract-valid state envelopes.

## Requirements

### Requirement: Deterministic v1 state execution order
The pipeline orchestrator SHALL execute states in this exact order: `fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> adf_block_ops -> merge_candidates -> patch -> verify -> publish`. The orchestrator MUST wire `adf_block_ops` output to `merge_candidates` input. The `patch` state MUST receive operations from `merge_candidates` output and no longer receive `md_assist_edit` or `adf_table_edit` outputs directly.

#### Scenario: All states succeed in order
- **WHEN** a run starts with valid input and no state returns a hard error
- **THEN** the orchestrator executes each state exactly once in the defined order and marks the run successful after `publish`

#### Scenario: AdfBlockOps output flows to MergeCandidates
- **WHEN** the orchestrator completes `adf_block_ops`
- **THEN** its output operations MUST be passed to `merge_candidates` as an input parameter

#### Scenario: Patch receives operations from merge only
- **WHEN** the orchestrator calls the patch state
- **THEN** it MUST pass `FetchOutput` and `MergeCandidatesOutput` (containing `Vec<Operation>`)
- **AND** it MUST NOT pass `MdAssistEditOutput` or `AdfTableEditOutput`

#### Scenario: State order mismatch is prevented
- **WHEN** a state transition is attempted out of the defined sequence
- **THEN** the orchestrator MUST fail the run with a deterministic transition error and MUST NOT execute downstream states

#### Scenario: Patch output is propagated to verify and publish
- **WHEN** `patch` produces updated `candidate_page_adf`
- **THEN** `verify` evaluates the updated candidate payload
- **AND** `publish` receives the same verified payload

#### Scenario: AdfBlockOps executes between AdfTableEdit and MergeCandidates
- **WHEN** the orchestrator reaches the `adf_block_ops` step
- **THEN** it MUST execute after `adf_table_edit` and before `merge_candidates`

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

### Requirement: Pipeline crate exports MultiPageOrchestrator
The `atlassy-pipeline` crate MUST export `MultiPageOrchestrator` alongside the existing `Orchestrator`. Both are public types available to CLI and other consumers.

#### Scenario: MultiPageOrchestrator is constructable
- **WHEN** a `MultiPageOrchestrator` is created with a `ConfluenceClient` and artifact root
- **THEN** it MUST be usable to run `MultiPageRequest` plans

#### Scenario: Existing Orchestrator unchanged
- **WHEN** `Orchestrator::run()` is called with a single `RunRequest`
- **THEN** behavior MUST be identical to pre-Phase-8 (no changes to per-page pipeline)

#### Scenario: MultiPageOrchestrator uses Orchestrator internally
- **WHEN** `MultiPageOrchestrator` processes a page
- **THEN** it MUST delegate to `Orchestrator::run()` for per-page pipeline execution
- **AND** it MUST NOT duplicate or reimplement per-page pipeline logic
