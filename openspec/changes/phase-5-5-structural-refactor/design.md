## Context

The pipeline currently represents patch operations using three separate types across two crates:

- `PatchCandidate { path, value }` in `atlassy-adf/src/lib.rs` — no `op` field, always implies replace.
- `PatchOperation { op: String, path, value }` in `atlassy-adf/src/lib.rs` — `op` is always `"replace"`.
- `PatchOp { op: String, path, value }` in `atlassy-contracts/src/types.rs` — serialization duplicate of `PatchOperation`.

Data flows through a conversion chain: `ProseChangeCandidate`/`TableChangeCandidate` → `PatchCandidate` → `PatchOperation` → `PatchOp`. Each conversion is a map with no logic, just field copying.

Phase 6 will add `Insert` and `Remove` operation kinds. Bolting these onto the existing type chain would compound the Shotgun Surgery smell — every new op kind would require changes in 10+ locations. The refactoring consolidates first.

The verify stage (`states/verify.rs`, 85 lines) handles three concerns in a single `if/else if/else` chain: forced test failure, table shape guards, and scope containment. Phase 6 will add a fourth concern (post-mutation structural validation). Extracting methods now prevents Divergent Change.

## Goals / Non-Goals

**Goals:**

- Consolidate `PatchCandidate`, `PatchOperation`, and `PatchOp` into a single `Operation` enum.
- Prepare the pipeline interface for Phase 6 (`block_ops` on `RunRequest`, `AdfBlockOps` state, ordering module).
- Extract verify check functions for independent testability.
- Maintain byte-identical behavior: all 159 tests pass, artifact JSON format preserved.

**Non-Goals:**

- Changing `MergeCandidatesOutput` or `VerifyInput` to carry `Vec<Operation>` — deferred to Phase 6 when op-type info is actually needed. Merge currently only sees paths, not values; changing it now would require restructuring the data flow without a use case.
- Replacing `ProseChangeCandidate` or `TableChangeCandidate` — these carry domain-specific fields and are still useful.
- Adding any new operation kinds — `Operation::Replace` is the only variant in Phase 5.5.
- Implementing operation ordering — the ordering module is a stub (identity sort).
- Implementing `AdfBlockOps` logic — the state is a no-op pass-through.

## Decisions

### D1: `Operation` enum lives in `atlassy-contracts`

`Operation` is used by `atlassy-adf` (validation/application logic) and `atlassy-pipeline` (state construction and output types). Placing it in `atlassy-contracts` — the shared types crate at the bottom of the dependency graph — avoids any new cross-crate dependencies.

Alternative considered: place in `atlassy-adf` where the behavior lives. Rejected because `PatchOutput` (in contracts) needs to reference the type, and contracts cannot depend on adf.

### D2: Use `#[serde(tag = "op", rename_all = "snake_case")]` for the enum

This makes `Operation::Replace { path, value }` serialize as `{"op": "replace", "path": "...", "value": ...}`, which is byte-identical to the current `PatchOp { op: "replace", path, value }`. Artifact compatibility is maintained without migration.

Alternative considered: `#[serde(untagged)]` — rejected because it loses the `op` field in JSON, breaking artifact format. Also considered external tagging (serde default) — rejected because it wraps values in `{"Replace": {...}}` which changes the JSON structure.

### D3: Keep `PatchOutput` field name as `patch_ops`

Renaming `patch_ops` to `operations` would cascade to `RunSummary.patch_ops_bytes`, orchestrator references, CLI KPI reporting, and test assertions (20+ references). The type change (`Vec<PatchOp>` → `Vec<Operation>`) is what matters for the refactoring; the field name is cosmetic.

Alternative considered: rename to `operations` for clarity. Deferred to avoid unnecessary blast radius in a zero-behavior-change refactoring.

### D4: Split `build_patch_ops` into `validate_operations` + `apply_operations`

The current `build_patch_ops` does two things: validates scope and constructs `PatchOperation`. With the `Operation` enum, construction happens at the call site (the pipeline state builds `Operation::Replace` directly). What remains is validation — a pure function.

`apply_patch_ops` becomes `apply_operations` with a `match` on operation variants. In Phase 5.5, only the `Replace` arm exists. Phase 6 adds `Insert` and `Remove` arms.

Alternative considered: keep `build_patch_ops` taking `Operation` as both input and output. Rejected because a function that returns its input unchanged (after validation) is misleading. Separating validation from application follows refactoring.guru's Separate Query from Modifier.

### D5: `AdfBlockOps` state positioned between `AdfTableEdit` and `MergeCandidates`

This position makes sense in the data flow: block operations are resolved after text-level edits (prose/table) and before merge collects all changes. The state receives `block_ops` from `RunRequest` and will produce `Operation` instances in Phase 6.

In Phase 5.5, the state is a no-op — it produces empty output. The orchestrator wires it but it doesn't affect the data flow.

### D6: Verify extraction preserves the existing check order

The three extracted functions are called in the same order as the current `if/else if/else` chain: `check_forced_fail` → `check_table_shape_integrity` → `check_scope_containment`. This ensures identical behavior. Each function returns `Option<VerifyResult>` — `Some(Fail)` to halt with a specific error, `None` to continue to the next check.

## Risks / Trade-offs

[Type change cascades to test assertions] → Tests reference `PatchOp`, `PatchCandidate`, and function names (`build_patch_ops`, `apply_patch_ops`). Mitigation: update tests in the same commit; run `cargo test --workspace` after each refactoring step.

[Serde compatibility across enum evolution] → Phase 6 will add `Insert` and `Remove` variants. With `#[serde(tag = "op")]`, old artifacts containing only `"replace"` ops will still deserialize correctly. New variants are additive. Risk is low.

[Ordering module stub creates dead code] → `sort_operations()` returns its input unchanged in Phase 5.5. Mitigation: mark with `#[allow(unused)]` if needed, or call it in the pipeline to keep it wired. Phase 6 replaces the stub quickly.

[MergeCandidatesOutput deferred change] → Phase 6 will need to change `MergeCandidatesOutput` from `changed_paths: Vec<String>` to carry `Vec<Operation>`. This requires restructuring how merge accesses candidate values (currently it only sees paths). Risk: the Phase 5.5 refactoring doesn't address this, so Phase 6 still has a data flow change to make. Mitigation: this is a deliberate deferral; the alternative (changing it now without a use case) risks Speculative Generality.

## Refactoring Sequence

Ordered for compile-test checkpoints. Each step should compile; all tests pass at the end.

1. Add `Operation` enum + `BlockOp`/`BlockOpKind` to `atlassy-contracts/src/types.rs` (additive).
2. Change `PatchOutput.patch_ops` type from `Vec<PatchOp>` to `Vec<Operation>`.
3. Drop `PatchOp` struct from contracts (now unused).
4. Replace `build_patch_ops` + `apply_patch_ops` with `validate_operations` + `apply_operations` in `atlassy-adf/src/patch.rs`.
5. Drop `PatchCandidate` + `PatchOperation` from `atlassy-adf/src/lib.rs` (now unused).
6. Update `atlassy-pipeline/src/states/patch.rs` to build `Operation::Replace` directly.
7. Add `AdfBlockOps` state + ordering module stub (new files).
8. Wire `AdfBlockOps` into orchestrator + state tracker.
9. Add `block_ops` to `RunRequest`, update CLI construction sites.
10. Extract verify check functions.
11. Update all test files.
12. `cargo test --workspace` — all 159 tests pass.
