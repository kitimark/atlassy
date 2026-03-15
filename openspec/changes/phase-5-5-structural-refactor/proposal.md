## Why

The codebase has three code smells (per refactoring.guru) that will compound when Phase 6 adds insert/delete operations: Primitive Obsession (`op: String` in patch types), Shotgun Surgery (the operation concept split across `PatchCandidate`, `PatchOperation`, and `PatchOp` in two crates), and Divergent Change (`verify.rs` handling too many concerns in one function). Refactoring first — with zero behavior change — makes Phase 6 feature work clean and sustainable. This follows the principle: "when you need to add a feature, refactor the code first to make adding the feature easy, then add the feature."

## What Changes

- **BREAKING**: Replace `PatchCandidate` (atlassy-adf), `PatchOperation` (atlassy-adf), and `PatchOp` (atlassy-contracts) with a single `Operation` enum in atlassy-contracts. Phase 5.5 introduces only `Operation::Replace { path, value }`.
- **BREAKING**: `build_patch_ops()` and `apply_patch_ops()` in atlassy-adf are replaced by `validate_operations()` and `apply_operations()`.
- **BREAKING**: `PatchOutput.patch_ops` type changes from `Vec<PatchOp>` to `Vec<Operation>`. Field name kept for serialization compatibility.
- Add `BlockOp` struct and `BlockOpKind` enum to atlassy-contracts (preparation for Phase 6, unused in Phase 5.5).
- Add `block_ops: Vec<BlockOp>` field to `RunRequest` (always empty in Phase 5.5).
- Add `AdfBlockOps` pipeline state between `AdfTableEdit` and `MergeCandidates` (no-op pass-through).
- Add `ordering.rs` module in atlassy-adf with stub `sort_operations()` (identity sort).
- Extract verify check functions from monolithic `if/else if/else` chain into focused functions: `check_forced_fail()`, `check_table_shape_integrity()`, `check_scope_containment()`.
- All 159 existing tests must pass with zero behavior change.

## Capabilities

### New Capabilities

- `operation-type-system`: The unified `Operation` enum that replaces three separate patch types, providing compile-time safety for operation kinds and eliminating primitive obsession.
- `block-ops-pipeline-state`: The `AdfBlockOps` pipeline state stub and `block_ops` field on `RunRequest`, preparing the pipeline interface for Phase 6 structural operations.
- `operation-ordering`: The ordering module stub in atlassy-adf that will hold reverse-document-order sorting logic in Phase 6.

### Modified Capabilities

- `patch-stage-candidate-application`: Patch stage now builds `Operation::Replace` directly instead of going through `PatchCandidate` → `build_patch_ops` → `PatchOperation` → `PatchOp` conversion chain. Functions renamed to `validate_operations` and `apply_operations`.
- `table-shape-change-guards`: Verify logic extracted into focused check functions. Same behavior, different internal structure.
- `pipeline-state-orchestration`: Pipeline state order gains `AdfBlockOps` between `AdfTableEdit` and `MergeCandidates`. `PipelineState` enum gains new variant.

## Impact

- **atlassy-contracts**: `Operation` enum added, `PatchOp` removed, `PatchOutput` type changed, `BlockOp`/`BlockOpKind` added.
- **atlassy-adf**: `PatchCandidate` and `PatchOperation` removed, `patch.rs` restructured, `ordering.rs` added.
- **atlassy-pipeline**: `states/patch.rs` rebuilt to use `Operation` directly, `states/verify.rs` extracted into check functions, `states/adf_block_ops.rs` added, orchestrator wires new state, `RunRequest` gains `block_ops` field, `PipelineState` gains `AdfBlockOps`.
- **atlassy-cli**: `RunRequest` construction sites add `block_ops: vec![]`.
- **atlassy-confluence**: No changes.
- **Tests**: 3 test files updated to use new types. No new test behavior — only type alignment.
- **Serialization**: `Operation::Replace` with `#[serde(tag = "op")]` produces byte-identical JSON to previous `PatchOp { op: "replace" }`. Artifact format preserved.
