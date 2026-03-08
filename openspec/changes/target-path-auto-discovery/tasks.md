## 1. Foundation types and constants

- [x] 1.1 Add `TargetRoute` enum (`Prose`, `TableCell`) to `crates/atlassy-adf/src/lib.rs` (public, with `Display` impl for error messages)
- [x] 1.2 Add `EDITABLE_PROSE_TYPES` public constant to `crates/atlassy-adf/src/lib.rs` with the 7-type whitelist: `paragraph`, `heading`, `bulletList`, `orderedList`, `listItem`, `blockquote`, `codeBlock`
- [x] 1.3 Add `AdfError::TargetDiscoveryFailed { route: String, index: usize, found: usize }` variant to `AdfError` enum in `crates/atlassy-adf/src/lib.rs`
- [x] 1.4 Add `ERR_TARGET_DISCOVERY_FAILED` constant to `crates/atlassy-contracts/src/lib.rs`

## 2. Discovery function

- [x] 2.1 Implement `discover_target_path()` in `crates/atlassy-adf/src/lib.rs` following the 7-step algorithm: collect text nodes from index, filter by scope, filter by route using `path_has_ancestor_type()`, sort lexicographically, index by `target_index`, append `/text`, return error if out of bounds
- [x] 2.2 Add unit test `discovers_first_prose_text_in_section` in `atlassy-adf`
- [x] 2.3 Add unit test `discovers_nth_prose_text_with_index` in `atlassy-adf`
- [x] 2.4 Add unit test `discovers_table_cell_text` in `atlassy-adf`
- [x] 2.5 Add unit test `discovery_respects_scope_boundary` in `atlassy-adf`
- [x] 2.6 Add unit test `discovery_fails_on_empty_section` in `atlassy-adf`
- [x] 2.7 Add unit test `discovery_fails_on_out_of_bounds_index` in `atlassy-adf`

## 3. RunMode type changes

- [x] 3.1 Change `target_path` from `String` to `Option<String>` in `RunMode::SimpleScopedProseUpdate` and `RunMode::SimpleScopedTableCellUpdate` in `crates/atlassy-pipeline/src/lib.rs`
- [x] 3.2 Fix all compilation errors from the `RunMode` type change — wrap existing `target_path` values in `Some(...)` at every construction site across all crates

## 4. Pipeline integration

- [x] 4.1 Add `target_index: usize` field to `RunRequest` in `crates/atlassy-pipeline/src/lib.rs` (default `0`)
- [x] 4.2 Insert discovery call in the `SimpleScopedProseUpdate` match arm of `run_md_assist_edit_state()` — when `target_path` is `None`, call `discover_target_path()` with `TargetRoute::Prose` and `request.target_index`, mapping errors via explicit `map_err` to `ERR_TARGET_DISCOVERY_FAILED` with state `MdAssistEdit`
- [x] 4.3 Insert discovery call in the `SimpleScopedTableCellUpdate` match arm of `run_adf_table_edit_state()` — when `target_path` is `None`, call `discover_target_path()` with `TargetRoute::TableCell` and `request.target_index`, mapping errors via explicit `map_err` to `ERR_TARGET_DISCOVERY_FAILED` with state `AdfTableEdit`
- [x] 4.4 Refactor `route_for_node()` in `crates/atlassy-pipeline/src/lib.rs` to reference `EDITABLE_PROSE_TYPES` from `atlassy-adf` instead of the inline match pattern

## 5. CLI and manifest changes

- [x] 5.1 Add `target_index: Option<u32>` field with `#[serde(default)]` to `ManifestRunEntry` in `crates/atlassy-cli/src/main.rs`
- [x] 5.2 Add `#[serde(default)]` to the `timestamp` field of `ManifestRunEntry`
- [x] 5.3 Update `run_mode_from_manifest()` to pass `None` for `target_path` when the field is absent (remove `unwrap_or_else` defaults) and pass `target_index` through to `RunRequest`
- [x] 5.4 Add `--target-index <N>` optional CLI argument and wire it to `RunRequest.target_index`

## 6. Telemetry

- [x] 6.1 Add `discovered_target_path: Option<String>` field with `#[serde(skip_serializing_if = "Option::is_none")]` to `RunSummary` in `crates/atlassy-contracts/src/lib.rs`
- [x] 6.2 Set `discovered_target_path` in the pipeline when auto-discovery resolves a path; leave `None` when explicit `target_path` is provided

## 7. Pipeline integration tests

- [x] 7.1 Add integration test `explicit_target_path_skips_discovery` in `atlassy-pipeline` — verify that providing `Some(path)` bypasses discovery and uses the explicit path
- [x] 7.2 Add integration test `pipeline_auto_discovers_and_patches` in `atlassy-pipeline` — end-to-end: omit `target_path`, auto-discover, verify patch succeeds with correct path

## 8. Verification

- [x] 8.1 Run `cargo test --workspace` and fix any failures
- [x] 8.2 Run `cargo clippy --workspace --all-targets -- -D warnings` and fix any warnings
- [x] 8.3 Verify existing manifests with explicit `target_path` produce identical behavior (backward compatibility)
