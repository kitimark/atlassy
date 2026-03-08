## MODIFIED Requirements

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
