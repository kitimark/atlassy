## Why

The CLI `run` command hardcodes `"2026-03-06T10:00:00Z"` as the timestamp for every `RunRequest`, and `default_manifest_timestamp()` uses `"1970-01-01T00:00:00Z"`. Every pipeline run receives an identical timestamp regardless of when it executes, making run ordering, deduplication, and audit trails unreliable. No time crate exists in the dependency tree to generate real timestamps.

## What Changes

- Add the `chrono` crate as a workspace dependency for UTC timestamp generation.
- Replace the hardcoded timestamp in `execute_run_command()` (`commands/run.rs:113`) with `Utc::now().to_rfc3339()`.
- Replace the static `"1970-01-01T00:00:00Z"` default in `default_manifest_timestamp()` (`types.rs:563`) with a real UTC timestamp at parse time.
- Preserve hardcoded timestamps in test files (`pipeline_integration.rs`, `contract_validation.rs`) since deterministic values are correct for tests.

## Capabilities

### New Capabilities

- `run-request-timestamping`: The requirement that `RunRequest.timestamp` and manifest default timestamps reflect actual UTC invocation time rather than hardcoded placeholders.

### Modified Capabilities

_(none — existing `pipeline-state-orchestration` requires timestamps be present in envelopes, which they already are; this change fixes their accuracy, not the contract)_

## Impact

- **Crates**: `atlassy-cli` (primary), workspace `Cargo.toml` (new dependency)
- **Dependencies**: Adds `chrono` with `clock` and `serde` features
- **Files**: `commands/run.rs`, `types.rs`, workspace `Cargo.toml`
- **Test files unchanged**: Hardcoded timestamps in test code remain deterministic
- **Risk**: Low — isolated to timestamp assignment at two call sites
