## Why

The pipeline can only replace text within existing ADF blocks. Phase 6 adds insert and delete operations for ADF blocks, enabling structural editing — the foundation for all Structural phases (6-9). Phase 5.5 established the `Operation` enum with `Replace` only and left the `AdfBlockOps` state as a no-op. Phase 6 activates both, wires the unified operation flow through merge → patch → verify, and implements the reverse-document-order algorithm.

## What Changes

- Add `Operation::Insert { parent_path, index, block }` and `Operation::Remove { target_path }` variants to the `Operation` enum.
- **BREAKING**: Refactor `BlockOp` from struct with `kind: BlockOpKind` (Primitive Obsession) to enum with `Insert { parent_path, index, block }` and `Remove { target_path }` variants. Drop `BlockOpKind`.
- **BREAKING**: `MergeCandidatesOutput` changes from `changed_paths: Vec<String>` to `operations: Vec<Operation>`. Merge becomes the operation collector — builds `Operation::Replace` from prose/table candidates (moved from patch) and receives block operations from `AdfBlockOps`.
- **BREAKING**: `PatchInput` and `VerifyInput` change to carry `Vec<Operation>` instead of `Vec<String>` changed paths.
- Activate `AdfBlockOps` state: translates `BlockOp` → `Operation`, validates scope, type, and position. Scope limited to `editable_prose` types (D-018).
- Replace identity stub in `ordering.rs` with reverse-document-order algorithm (D-020): replaces first, then structural ops grouped by parent path descending index, remove-before-insert at same index.
- Implement `apply_insert()` and `apply_remove()` in `atlassy-adf/src/patch.rs`.
- Add `check_structural_validity()` in `atlassy-adf` for post-mutation ADF validation (non-empty doc.content, valid block types, parent-child rules, heading attrs).
- Add op-aware `check_operation_legality()` in verify: allows intentional structural changes from declared operations, blocks unintended mutations.
- Simplify `states/patch.rs`: receives operations from merge instead of building them from prose/table candidates. No longer needs `md_edit` or `table_edit` parameters.
- Add 3 error codes: `InsertPositionInvalid`, `RemoveAnchorMissing`, `PostMutationSchemaInvalid`.

## Capabilities

### New Capabilities

- `block-insert-remove`: Insert and Remove operation variants, apply logic (`apply_insert`, `apply_remove`), scope/type validation, and position checking for editable_prose block types.
- `reverse-document-order-sorting`: Real ordering algorithm replacing the identity stub — partitions replaces vs structural, groups by parent path, descending index, remove-before-insert at same index, conflict detection.
- `structural-validity-check`: Post-mutation ADF validation ensuring doc.content is non-empty, all blocks have valid types, parent-child relationships are correct, headings have attrs.level.
- `operation-aware-verification`: Op-type-aware verify checks that distinguish intentional structural changes from accidental mutations, with Insert/Remove-specific validation.
- `unified-operation-merge`: Merge state becomes the operation collector — builds all Operation types from edit state candidates and block ops, performs cross-route conflict detection on operations.

### Modified Capabilities

- `operation-type-system`: Operation enum gains Insert and Remove variants. BlockOp refactored from struct to enum (Replace Type Code with Strategy).
- `block-ops-pipeline-state`: Activated from no-op — translates BlockOp to Operation, validates scope/type/position, output wired to merge.
- `operation-ordering`: Identity stub replaced with reverse-document-order algorithm.
- `patch-stage-candidate-application`: Simplified — receives operations from merge, no longer builds Replace ops from candidates. Patch params lose md_edit and table_edit.
- `pipeline-state-orchestration`: AdfBlockOps output wired to merge. Patch params simplified.
- `table-shape-change-guards`: Integrated into op-aware verify chain alongside new structural checks.
- `typed-error-codes`: Three new error codes added.

## Impact

- **atlassy-contracts**: `Operation` gains 2 variants, `BlockOp` refactored to enum, `BlockOpKind` removed, `MergeCandidatesOutput`/`PatchInput`/`VerifyInput` change from paths to operations, 3 new error codes.
- **atlassy-adf**: `patch.rs` gains `apply_insert`/`apply_remove`, `ordering.rs` gains real algorithm, new `structural_validity.rs` module, new `AdfError` variants.
- **atlassy-pipeline**: `merge_candidates.rs` restructured as operation collector, `patch.rs` simplified, `verify.rs` gains 2 new check functions, `adf_block_ops.rs` activated, orchestrator wiring updated.
- **atlassy-cli**: `BlockOp` construction sites updated for new enum shape.
- **atlassy-confluence**: No changes.
