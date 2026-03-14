## Why

The target path auto-discovery feature is fully implemented but has three minor gaps identified during code review: (1) the `SimpleScopedUpdate` RunMode variant is dead code — unused in integration tests, QA manifests, or production workflows, and fully superseded by route-specific variants (`SimpleScopedProseUpdate`, `SimpleScopedTableCellUpdate`); (2) the `to_hard_error()` fallthrough maps `AdfError::TargetDiscoveryFailed` to `ErrorCode::SchemaInvalid` instead of `ErrorCode::TargetDiscoveryFailed` — a dead code arm that is actively misleading to future contributors; (3) there is no integration test for table cell auto-discovery (`target_path: None` with `SimpleScopedTableCellUpdate`), creating an asymmetry with the prose route which has full integration coverage.

## What Changes

- **Remove `RunMode::SimpleScopedUpdate` variant** and all associated match arms in pipeline states, CLI command mapping, and manifest mode parsing. This eliminates the duplicated magic string `"/content/1/content/0/text"` and the ambiguous-route mode that cannot support auto-discovery. Refactoring pattern: Remove Dead Code (Dispensable), eliminates Duplicate Code and Primitive Obsession smells.
- **Fix `to_hard_error()` error code mapping** so `AdfError::TargetDiscoveryFailed` maps to `ErrorCode::TargetDiscoveryFailed` instead of `ErrorCode::SchemaInvalid`. This arm remains unreachable at runtime (discovery errors use explicit `map_err`) but becomes self-documenting and defensively correct. Refactoring pattern: Replace Magic Number with Symbolic Constant.
- **Add table cell auto-discovery integration test** to verify `SimpleScopedTableCellUpdate` with `target_path: None` works end-to-end through all 9 pipeline stages, matching the existing prose route integration test coverage.

## Capabilities

### New Capabilities

*(none — this change addresses gaps in existing capabilities)*

### Modified Capabilities

- `target-path-auto-discovery`: Remove requirement that `SimpleScopedUpdate` keeps `target_path` as `String`, since the variant is being removed entirely. Add integration test scenario for table cell auto-discovery end-to-end.
- `typed-error-codes`: Change requirement that "remaining ADF variants map to SchemaInvalid" to exclude `TargetDiscoveryFailed`, which now maps to its own error code in `to_hard_error()`.

## Impact

- **`atlassy-pipeline`**: `RunMode` enum loses `SimpleScopedUpdate` variant; match arms removed in `md_assist_edit.rs` and `adf_table_edit.rs`; one match arm changed in `error_map.rs`; one integration test added.
- **`atlassy-cli`**: `ManifestMode` enum loses `SimpleScopedUpdate`; `CliMode` enum loses `SimpleScopedUpdate`; match arms removed in `manifest.rs`, `commands/run.rs`, `main.rs`.
- **No API or contract changes** — `SimpleScopedUpdate` was never exposed externally. Manifest entries using `simple_scoped_update` mode would fail at deserialization, but no such manifests exist.
