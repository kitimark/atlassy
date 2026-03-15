## 1. Operation Enum + BlockOp Refactor (atlassy-contracts)

- [ ] 1.1 Add `Operation::Insert { parent_path: String, index: usize, block: Value }` variant to the `Operation` enum
- [ ] 1.2 Add `Operation::Remove { target_path: String }` variant to the `Operation` enum
- [ ] 1.3 Refactor `BlockOp` from struct to enum with `Insert { parent_path, index, block }` and `Remove { target_path }` variants
- [ ] 1.4 Remove `BlockOpKind` enum
- [ ] 1.5 Change `MergeCandidatesOutput` from `changed_paths: Vec<String>` to `operations: Vec<Operation>`
- [ ] 1.6 Change `MergeCandidatesInput` to add `block_operations: Vec<Operation>` field
- [ ] 1.7 Change `PatchInput` to carry `operations: Vec<Operation>` instead of `changed_paths: Vec<String>`
- [ ] 1.8 Change `VerifyInput` to carry `operations: Vec<Operation>` instead of `changed_paths: Vec<String>`

## 2. Error Codes (atlassy-contracts)

- [ ] 2.1 Add `InsertPositionInvalid` variant to `ErrorCode` with `as_str` returning `"ERR_INSERT_POSITION_INVALID"`
- [ ] 2.2 Add `RemoveAnchorMissing` variant to `ErrorCode` with `as_str` returning `"ERR_REMOVE_ANCHOR_MISSING"`
- [ ] 2.3 Add `PostMutationSchemaInvalid` variant to `ErrorCode` with `as_str` returning `"ERR_POST_MUTATION_SCHEMA_INVALID"`
- [ ] 2.4 Update `ErrorCode::ALL` array and test coverage for all 3 new variants

## 3. ADF Error Variants (atlassy-adf)

- [ ] 3.1 Add `InsertPositionInvalid(String)` variant to `AdfError`
- [ ] 3.2 Add `RemoveTargetNotFound(String)` variant to `AdfError`
- [ ] 3.3 Add `PostMutationInvalid(String)` variant to `AdfError`
- [ ] 3.4 Add `OperationConflict(String)` variant to `AdfError` (for remove-prefix conflicts)

## 4. Apply Insert and Remove (atlassy-adf/src/patch.rs)

- [ ] 4.1 Add `Operation::Insert` arm to `validate_operations()` — check parent_path in scope, not whole-body
- [ ] 4.2 Add `Operation::Remove` arm to `validate_operations()` — check target_path in scope, not whole-body
- [ ] 4.3 Implement `apply_insert()` helper — navigate to parent_path, get array, insert block at index. Return `InsertPositionInvalid` if parent is not an array or index is out of bounds
- [ ] 4.4 Implement `apply_remove()` helper — parse target_path into (parent_path, index), navigate to parent, remove element. Return `RemoveTargetNotFound` if path doesn't resolve
- [ ] 4.5 Add `Operation::Insert` and `Operation::Remove` arms to `apply_operations()` calling the helpers
- [ ] 4.6 Add `split_parent_index()` utility — parses `/content/3` into `("/content", 3)`

## 5. Reverse-Document-Order Sorting (atlassy-adf/src/ordering.rs)

- [ ] 5.1 Implement `extract_path_info()` — for each Operation, extract (parent_path, index, op_kind) for sorting
- [ ] 5.2 Implement partition: separate Replace ops (leaf-path) from structural ops (Insert/Remove)
- [ ] 5.3 Implement group-by-parent-path with descending index sort within groups
- [ ] 5.4 Implement same-index tie-breaking: Remove before Insert
- [ ] 5.5 Implement conflict detection: reject if any Remove path is a prefix of another operation's path
- [ ] 5.6 Return sorted operations: replaces first, then structural ops in reverse document order

## 6. Structural Validity (atlassy-adf — new module)

- [ ] 6.1 Create `atlassy-adf/src/structural_validity.rs` with `check_structural_validity(adf: &Value) -> Result<(), AdfError>`
- [ ] 6.2 Implement check: `doc.content` is a non-empty array
- [ ] 6.3 Implement check: every element in `doc.content` has a `type` field
- [ ] 6.4 Implement check: `heading` nodes have `attrs.level` in range 1-6
- [ ] 6.5 Add `mod structural_validity` and `pub use` to `atlassy-adf/src/lib.rs`

## 7. AdfBlockOps Activation (atlassy-pipeline)

- [ ] 7.1 Implement `BlockOp::Insert` → `Operation::Insert` translation in `adf_block_ops.rs`
- [ ] 7.2 Implement `BlockOp::Remove` → `Operation::Remove` translation in `adf_block_ops.rs`
- [ ] 7.3 Add scope validation: check parent_path/target_path against `allowed_scope_paths` (needs fetch output access)
- [ ] 7.4 Add block type validation: check inserted block type is in `EDITABLE_PROSE_TYPES`
- [ ] 7.5 Update `run_adf_block_ops_state` signature to accept `FetchOutput` for scope paths

## 8. Merge Restructure (atlassy-pipeline/src/states/merge_candidates.rs)

- [ ] 8.1 Add `adf_block_ops` output as parameter to `run_merge_candidates_state`
- [ ] 8.2 Build `Operation::Replace` from `ProseChangeCandidate` (move from patch.rs)
- [ ] 8.3 Build `Operation::Replace` from `TableChangeCandidate` (move from patch.rs)
- [ ] 8.4 Collect block operations from `AdfBlockOpsOutput.operations`
- [ ] 8.5 Update cross-route conflict detection to work with operations (extract paths from ops)
- [ ] 8.6 Output `MergeCandidatesOutput { operations: Vec<Operation> }`

## 9. Patch Simplification (atlassy-pipeline/src/states/patch.rs)

- [ ] 9.1 Remove `md_edit` and `table_edit` parameters from `run_patch_state`
- [ ] 9.2 Receive operations from `merged.payload.operations` instead of building them
- [ ] 9.3 Update `PatchInput` construction to use `operations` field
- [ ] 9.4 Verify `validate_operations` → `sort_operations` → `apply_operations` chain works with mixed op types

## 10. Verify Op-Awareness (atlassy-pipeline/src/states/verify.rs)

- [ ] 10.1 Update `VerifyInput` construction to use `operations` from merge output
- [ ] 10.2 Add `check_operation_legality()` function — per-operation type validation
- [ ] 10.3 Add `check_structural_validity()` call — conditional on presence of Insert/Remove ops
- [ ] 10.4 Update `check_table_shape_integrity` to extract paths from operations
- [ ] 10.5 Update `check_scope_containment` to extract paths from operations
- [ ] 10.6 Wire new checks into the verify chain in order: forced_fail → table_shape → operation_legality → scope → structural_validity

## 11. Orchestrator Wiring (atlassy-pipeline/src/orchestrator.rs)

- [ ] 11.1 Pass `FetchOutput` to `run_adf_block_ops_state`
- [ ] 11.2 Pass `AdfBlockOpsOutput` to `run_merge_candidates_state`
- [ ] 11.3 Pass `MdAssistEditOutput` and `AdfTableEditOutput` to merge (for Replace construction)
- [ ] 11.4 Remove `md_edit` and `table_edit` from `run_patch_state` call
- [ ] 11.5 Update summary population for new operation metrics

## 12. CLI Updates (atlassy-cli)

- [ ] 12.1 Update `BlockOp` construction sites in `run.rs` for new enum shape
- [ ] 12.2 Update `BlockOp` construction sites in `run_batch.rs` for new enum shape

## 13. Unit Tests (atlassy-adf)

- [ ] 13.1 Test `apply_insert` — valid insert at beginning, middle, end of array
- [ ] 13.2 Test `apply_insert` — out-of-bounds index, non-array parent, empty path
- [ ] 13.3 Test `apply_remove` — valid remove, non-existent path, last element removal
- [ ] 13.4 Test `sort_operations` — replaces before structural, descending index, remove-before-insert tie-break
- [ ] 13.5 Test `sort_operations` — conflict detection (remove prefix of another op)
- [ ] 13.6 Test `sort_operations` — operations across different parents
- [ ] 13.7 Test `check_structural_validity` — valid ADF, empty content, missing type, bad heading level
- [ ] 13.8 Test `validate_operations` — Insert/Remove scope checks, whole-body rewrite rejection

## 14. Integration Tests (atlassy-pipeline)

- [ ] 14.1 Test insert-only run: single block insert produces correct ADF and publishes
- [ ] 14.2 Test remove-only run: single block remove produces correct ADF and publishes
- [ ] 14.3 Test mixed run: insert + replace + remove in same batch produces correct result
- [ ] 14.4 Test backward compatibility: replace-only run produces identical results to pre-Phase-6
- [ ] 14.5 Test error cases: out-of-scope insert, scope anchor remove, out-of-bounds index
- [ ] 14.6 Update existing integration tests for new merge output type and patch signature

## 15. Contract + Serialization Tests

- [ ] 15.1 Update `contract_validation.rs` for new `MergeCandidatesOutput`, `PatchInput`, `VerifyInput` shapes
- [ ] 15.2 Test `Operation::Insert` and `Operation::Remove` serde round-trip
- [ ] 15.3 Test `BlockOp` enum serde round-trip
- [ ] 15.4 Verify `Operation::Replace` serialization is unchanged (backward compatible)

## 16. Final Validation

- [ ] 16.1 Run `cargo test --workspace` — all tests pass
- [ ] 16.2 Run `cargo clippy --workspace` — zero warnings
- [ ] 16.3 Verify `BlockOpKind` does not appear anywhere in source code
- [ ] 16.4 Verify patch state no longer imports `MdAssistEditOutput` or `AdfTableEditOutput`
