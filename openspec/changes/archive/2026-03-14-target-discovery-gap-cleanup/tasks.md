## 1. Remove SimpleScopedUpdate (Dead Code)

- [x] 1.1 Remove `SimpleScopedUpdate` variant from `RunMode` enum in `crates/atlassy-pipeline/src/lib.rs`
- [x] 1.2 Remove `SimpleScopedUpdate` match arm from `run_md_assist_edit_state()` in `crates/atlassy-pipeline/src/states/md_assist_edit.rs` (lines 48-63)
- [x] 1.3 Remove `SimpleScopedUpdate` match arm from `run_adf_table_edit_state()` in `crates/atlassy-pipeline/src/states/adf_table_edit.rs` (lines 48-61)
- [x] 1.4 Remove `SimpleScopedUpdate` variant from `ManifestMode` enum in `crates/atlassy-cli/src/types.rs`
- [x] 1.5 Remove `SimpleScopedUpdate` match arm from `run_mode_from_manifest()` in `crates/atlassy-cli/src/manifest.rs` (lines 40-51)
- [x] 1.6 Remove `SimpleScopedUpdate` variant from `CliMode` enum and its match arm in `crates/atlassy-cli/src/main.rs`
- [x] 1.7 Remove `"simple-scoped-update"` match arm from `execute_run_command()` in `crates/atlassy-cli/src/commands/run.rs` (lines 91-94)
- [x] 1.8 Verify `cargo check --workspace` passes with no compilation errors

## 2. Fix to_hard_error() Fallthrough Mapping

- [x] 2.1 Separate `AdfError::TargetDiscoveryFailed { .. }` from the `SchemaInvalid` match arm group in `crates/atlassy-pipeline/src/error_map.rs` (line 60)
- [x] 2.2 Map `AdfError::TargetDiscoveryFailed { .. }` to `ErrorCode::TargetDiscoveryFailed` as its own match arm
- [x] 2.3 Verify `cargo check --workspace` passes

## 3. Add Table Cell Auto-Discovery Integration Test

- [x] 3.1 Add `pipeline_auto_discovers_table_cell_and_patches` test to `crates/atlassy-pipeline/tests/pipeline_integration.rs` using `table_allowed_cell_update_adf.json` fixture with `RunMode::SimpleScopedTableCellUpdate { target_path: None, text: "..." }`
- [x] 3.2 Assert `summary.success == true`, `summary.discovered_target_path == Some("/content/1/content/0/content/0/content/0/content/0/text")`, and `summary.applied_paths` matches the discovered path
- [x] 3.3 Verify the new test passes with `cargo test -p atlassy-pipeline`

## 4. Final Verification

- [x] 4.1 Run `make test` — all existing and new tests pass
- [x] 4.2 Run `make lint` — no clippy warnings
- [x] 4.3 Run `make fmt-check` — formatting is clean
