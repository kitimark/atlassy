## 1. Operation Type System (atlassy-contracts)

- [x] 1.1 Add `Operation` enum with `Replace { path: String, value: Value }` variant to `atlassy-contracts/src/types.rs`, using `#[serde(tag = "op", rename_all = "snake_case")]`
- [x] 1.2 Add `BlockOp` struct and `BlockOpKind` enum (`Insert`, `Remove`) to `atlassy-contracts/src/types.rs`
- [x] 1.3 Change `PatchOutput.patch_ops` type from `Vec<PatchOp>` to `Vec<Operation>`
- [x] 1.4 Remove `PatchOp` struct from `atlassy-contracts/src/types.rs`
- [x] 1.5 Update `atlassy-contracts/tests/contract_validation.rs` to use `Operation::Replace` in `PatchOutput` construction

## 2. Patch Module Restructure (atlassy-adf)

- [x] 2.1 Replace `build_patch_ops()` with `validate_operations()` in `atlassy-adf/src/patch.rs` — takes `&[Operation]` and `&[String]` scope paths, returns `Result<(), AdfError>`
- [x] 2.2 Replace `apply_patch_ops()` with `apply_operations()` in `atlassy-adf/src/patch.rs` — takes `&Value` and `&[Operation]`, matches on `Operation::Replace`, returns `Result<Value, AdfError>`
- [x] 2.3 Remove `PatchCandidate` and `PatchOperation` structs from `atlassy-adf/src/lib.rs`
- [x] 2.4 Update public re-exports in `atlassy-adf/src/lib.rs` (remove old, export new)
- [x] 2.5 Update `atlassy-adf/tests/patch_ops.rs` to use `Operation::Replace` instead of `PatchCandidate`/`PatchOperation`

## 3. Ordering Module (atlassy-adf)

- [x] 3.1 Create `atlassy-adf/src/ordering.rs` with `sort_operations()` stub (identity sort — returns input unchanged)
- [x] 3.2 Add `mod ordering` and `pub use ordering::*` to `atlassy-adf/src/lib.rs`

## 4. Pipeline Patch State Update (atlassy-pipeline)

- [x] 4.1 Update `atlassy-pipeline/src/states/patch.rs` to build `Operation::Replace` directly from prose and table change candidates (remove `PatchCandidate` intermediary)
- [x] 4.2 Call `validate_operations()` then `sort_operations()` then `apply_operations()` in the patch state
- [x] 4.3 Map `Vec<Operation>` into `PatchOutput.patch_ops` field (no conversion — direct assignment)
- [x] 4.4 Update imports to reference `Operation`, `validate_operations`, `apply_operations`, `sort_operations` from `atlassy-adf` and `atlassy-contracts`

## 5. AdfBlockOps Pipeline State (atlassy-pipeline)

- [x] 5.1 Add `AdfBlockOps` variant to `PipelineState` enum in `atlassy-contracts/src/types.rs`
- [x] 5.2 Update `PipelineState::ORDER` and `expected_next()` to place `AdfBlockOps` between `AdfTableEdit` and `MergeCandidates`
- [x] 5.3 Create `atlassy-pipeline/src/states/adf_block_ops.rs` as a no-op pass-through state that persists empty artifacts
- [x] 5.4 Add `pub mod adf_block_ops` to `atlassy-pipeline/src/states/mod.rs`
- [x] 5.5 Wire `run_adf_block_ops_state()` call into orchestrator between `adf_table_edit` and `merge_candidates`

## 6. RunRequest Interface (atlassy-pipeline + atlassy-cli)

- [x] 6.1 Add `block_ops: Vec<BlockOp>` field to `RunRequest` in `atlassy-pipeline/src/lib.rs`
- [x] 6.2 Update `RunRequest` construction in `atlassy-cli/src/commands/run.rs` — add `block_ops: vec![]`
- [x] 6.3 Update `RunRequest` construction in `atlassy-cli/src/commands/run_batch.rs` — add `block_ops: vec![]`
- [x] 6.4 Update all `RunRequest` construction sites in test files — add `block_ops: vec![]`

## 7. Verify Extraction (atlassy-pipeline)

- [x] 7.1 Extract `check_forced_fail()` function from verify `if/else if/else` chain — returns `Option<VerifyResult>`
- [x] 7.2 Extract `check_table_shape_integrity()` function — returns `Option<VerifyResult>` with `ERR_TABLE_SHAPE_CHANGE` diagnostic
- [x] 7.3 Extract `check_scope_containment()` function — returns `Option<VerifyResult>` with `ERR_OUT_OF_SCOPE_MUTATION` diagnostic
- [x] 7.4 Rewrite `run_verify_state()` main body to call the three extracted functions in order

## 8. Integration Test Updates

- [x] 8.1 Update `atlassy-pipeline/tests/pipeline_integration.rs` to reference `Operation` in patch output assertions
- [x] 8.2 Verify `patch_ops` JSON artifact key is preserved (serde compatibility)

## 9. Final Validation

- [x] 9.1 Run `cargo test --workspace` — all 159 tests must pass
- [x] 9.2 Run `cargo clippy --workspace` — zero warnings
- [x] 9.3 Verify `PatchCandidate`, `PatchOperation`, `PatchOp` do not appear anywhere in source code
- [x] 9.4 Verify `build_patch_ops` and `apply_patch_ops` do not appear anywhere in source code
