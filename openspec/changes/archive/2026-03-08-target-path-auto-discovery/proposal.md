## Why

The KPI experiment workflow requires operators to manually discover `target_path` values by running `jq` path inspection on fetched ADF, selecting valid text nodes, and hardcoding them into batch manifests. These paths are fragile (tied to a specific page version), create manual overhead before every experiment batch, and encode runtime state into static configuration. The 2026-03-08 KPI revalidation demonstrated this directly: baseline runs advanced pages to version 6, invalidating all previously discovered paths and requiring re-discovery before the next batch could run.

## What Changes

- New `discover_target_path()` function in `atlassy-adf` that auto-selects the first (or Nth) valid text node within allowed scope, filtered by route (prose vs table cell).
- New `TargetRoute` enum (`Prose`, `TableCell`) in `atlassy-adf` to parameterize discovery.
- New `EDITABLE_PROSE_TYPES` shared constant in `atlassy-adf`, replacing the inline whitelist in `route_for_node()`.
- New `AdfError::TargetDiscoveryFailed` error variant with route, index, and candidate count.
- New `ERR_TARGET_DISCOVERY_FAILED` error constant in `atlassy-contracts`.
- **BREAKING**: `RunMode::SimpleScopedProseUpdate` and `RunMode::SimpleScopedTableCellUpdate` change `target_path` from `String` to `Option<String>`. All existing call sites that construct these variants must wrap in `Some(...)`.
- `RunRequest` gains a `target_index: usize` field (default 0) for selecting which discovered candidate to target.
- `ManifestRunEntry` gains `target_index: Option<u32>` and `timestamp` becomes optional (`#[serde(default)]`).
- `run_mode_from_manifest()` stops defaulting `target_path` when absent — passes `None` through to trigger auto-discovery.
- New `--target-index` CLI argument.
- `discovered_target_path` field added to run summary telemetry output.

## Capabilities

### New Capabilities

- `target-path-auto-discovery`: Runtime discovery of valid edit target paths from the ADF node path index and scope boundary. Covers the discovery function, route filtering, index-based selection, error handling, and integration into the pipeline edit states.

### Modified Capabilities

- `scoped-adf-fetch-and-index`: Adding a public constant (`EDITABLE_PROSE_TYPES`) and a public discovery function that consumes the node path index and allowed scope paths produced by this capability. The existing index structure and scope resolution requirements are unchanged — this adds a new consumer function to the same module.

## Impact

- **`crates/atlassy-adf/src/lib.rs`**: New function, enum, constant, and error variant.
- **`crates/atlassy-pipeline/src/lib.rs`**: `RunMode` type change (breaking for variant constructors), discovery calls in `run_md_assist_edit_state()` and `run_adf_table_edit_state()`, `route_for_node()` refactored to use shared constant.
- **`crates/atlassy-contracts/src/lib.rs`**: New error constant.
- **`crates/atlassy-cli/src/main.rs`**: `ManifestRunEntry` field additions, `run_mode_from_manifest()` behavior change, new CLI arg.
- **`qa/manifests/`**: Existing manifests with explicit `target_path` continue to work unchanged. New auto-discovery manifests (example already exists at `kpi-revalidation-auto-discovery.example.json`) omit `target_path`.
- **`qa/confluence-sandbox-test-plan.md`**: Already contains auto-discovery validation steps (Step 2b, lines 333-378).
