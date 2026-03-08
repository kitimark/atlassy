## Why

Error codes across the workspace are string constants (`&str`) compared via string equality. `PipelineError::Hard.code` is a `String` field, meaning any typo or refactor silently introduces a new error class. The pipeline modularization (Phase 2 of the code quality roadmap) is next — extracting `error_map.rs` from the pipeline monolith. That module should be born with typed error codes, not cement string-based codes into a new file.

## What Changes

- **BREAKING**: Replace 12 `&str` error code constants in `atlassy-contracts/src/constants.rs` with an `ErrorCode` enum.
- **BREAKING**: Change `PipelineError::Hard.code` from `String` to `ErrorCode`.
- Add `Display` and `Serialize` impls on `ErrorCode` that produce identical string representations (e.g., `ErrorCode::ScopeMiss` displays as `"ERR_SCOPE_MISS"`).
- Update pipeline code to construct `PipelineError::Hard` with `ErrorCode` variants instead of `ERR_*.to_string()`.
- Add `.to_string()` bridge in CLI where it currently consumes the `String` field directly — temporary until Phase 3 (CLI modularization) introduces typed matching.

## Capabilities

### New Capabilities

- `typed-error-codes`: Define the `ErrorCode` enum, its variants, serialization, and the contract that error codes are a closed set with stable string representations.

### Modified Capabilities

- `pipeline-state-orchestration`: Hard error halting now carries `ErrorCode` instead of `String` in the `code` field. The requirement for "deterministically mapped error code" becomes compiler-enforced rather than convention-enforced.

## Impact

- **`atlassy-contracts`**: `constants.rs` loses 12 `&str` constants, gains `ErrorCode` enum. `types.rs` may need adjustment if `PipelineError` types reference the code field.
- **`atlassy-pipeline`**: All `PipelineError::Hard { code: ERR_*.to_string(), .. }` sites change to `PipelineError::Hard { code: ErrorCode::*, .. }`. The `to_hard_error` function signature narrows from `impl Display` to `AdfError`, enabling direct variant matching (pulled forward from Phase 4).
- **`atlassy-cli`**: Downstream consumers that match on `code` as `&str` add `.to_string()` as a temporary bridge. No logic changes — just type adaptation.
- **No output schema changes**: `RunSummary.error_codes` remains `Vec<String>` (populated via `ErrorCode::to_string()`). JSON output is identical.
