## Context

Phase 5.5 established the `Operation` enum with `Replace` only, the `AdfBlockOps` no-op state, `block_ops` on `RunRequest`, extracted verify check functions, and the `ordering.rs` identity stub. The pipeline currently has a disconnected data flow: `AdfBlockOps` output is computed but never consumed — merge only receives prose/table paths, and patch builds `Operation::Replace` internally from prose/table candidates.

Phase 6 must wire the unified operation flow, add Insert/Remove variants, and implement the structural editing logic. The design must support Phases 7-9 without further pipeline restructuring (Open/Closed Principle).

## Goals / Non-Goals

**Goals:**

- Add `Insert` and `Remove` to `Operation` enum with per-variant data.
- Wire a single `Vec<Operation>` through merge → patch → verify (unified operation channel).
- Implement reverse-document-order sorting for correct multi-op batches.
- Implement post-mutation ADF structural validation.
- Scope insert/remove to `editable_prose` types only (D-018).
- All Foundation text-replacement functionality unchanged (backward compatible).

**Non-Goals:**

- Section-level composition (Phase 7) — inserting heading + body as a unit.
- Table creation or container insert/delete (Phase 7) — beyond editable_prose scope.
- Multi-page orchestration (Phase 8).
- Table topology operations (Phase 9).
- Stable-ID node addressing — reverse-order processing is sufficient for Phase 6.

## Decisions

### D1: Merge becomes the operation collector (Move Method)

Currently, `states/patch.rs` builds `Operation::Replace` from `ProseChangeCandidate` and `TableChangeCandidate`, then validates and applies. This responsibility splits across merge and patch. Phase 6 moves operation construction into merge:

- Merge builds `Operation::Replace` from prose/table candidates (logic moves from patch).
- Merge receives `adf_block_ops.payload.operations` (Insert/Remove from block ops).
- Merge outputs unified `Vec<Operation>`.
- Merge performs cross-route conflict detection on operations (extracting paths from each operation for collision checks).

This follows Single Responsibility: merge collects and validates the operation set, patch applies it.

Alternative considered: keep merge path-only and wire block_ops directly to patch. Rejected because patch would then do too many things (build Replace, receive block ops, merge, validate, sort, apply) — Divergent Change smell.

### D2: Patch simplifies to validate → sort → apply

After D1, patch receives `Vec<Operation>` from merge. It no longer needs `md_edit` or `table_edit` parameters. The function signature shrinks from 7 params to 4: `(artifact_store, request, tracker, fetch, merged)`.

This is the refactoring.guru Move Method technique — operation construction moves to where the source data lives (merge has access to all candidate sources), and application stays where the ADF is available (patch).

### D3: BlockOp becomes an enum (Replace Type Code with Strategy)

Current `BlockOp` is a struct with `kind: BlockOpKind, path: String, value: Option<Value>` — Primitive Obsession. Replace with an enum where each variant carries exactly its data:

```
enum BlockOp {
    Insert { parent_path: String, index: usize, block: Value },
    Remove { target_path: String },
}
```

`BlockOpKind` is removed. The compiler enforces that Insert has a block and index, Remove has only a path. `BlockOp` is the external request type; `AdfBlockOps` translates it to `Operation` (Command pattern: request → command).

### D4: Operation::Insert and Operation::Remove design

```
Operation::Insert {
    parent_path: String,    // JSON pointer to parent array (e.g., "/content")
    index: usize,           // position within parent array
    block: Value,           // complete ADF block node
}

Operation::Remove {
    target_path: String,    // JSON pointer to the block (e.g., "/content/3")
}
```

Phase 7 builds on these: "insert section" = multiple `Insert` ops (heading + paragraphs). Phase 9 may add table-specific variants (`InsertRow`, `RemoveColumn`). The generic `Insert`/`Remove` serve as the foundation.

### D5: Reverse-document-order sorting algorithm

Replace identity stub with:
1. Partition: replaces (leaf-path ops) vs structural (Insert/Remove).
2. Replaces first (they don't shift indices).
3. Structural ops: group by parent path, sort within group by descending index.
4. Same-index tie-break: Remove before Insert.
5. Pre-sort validation: reject if a Remove path is a prefix of another op's path (conflict).

All indices are relative to the original document. Processing highest-index-first prevents cascading index shifts.

### D6: Structural validity in atlassy-adf

Post-mutation validation as a pure function in `atlassy-adf` (domain logic, not pipeline logic):
- `doc.content` is a non-empty array.
- Every element has a `type` field.
- `type` is a known ADF block-level type.
- Parent-child validity: doc.content may only contain block nodes.
- `heading` nodes have `attrs.level` (1-6).

This is a structural rule check, not full JSON Schema validation. Confluence's publish endpoint serves as the ultimate backstop.

### D7: Op-aware verification

The verify chain gains two new functions:
- `check_operation_legality(operations, node_path_index, allowed_scope_paths)` — per-operation validation: Replace path in scope; Insert parent_path in scope, block type is editable_prose, index within bounds; Remove target_path in scope, not a scope anchor heading.
- `check_structural_validity(candidate_adf)` — post-mutation ADF validation (D6).

The existing `check_table_shape_integrity` and `check_scope_containment` operate on `changed_paths` which are now derived from operations. The full chain: `check_forced_fail → check_table_shape_integrity → check_operation_legality → check_scope_containment → check_structural_validity`.

## Risks / Trade-offs

[Merge restructure is the biggest change] → Moving Replace construction from patch to merge touches both states and the orchestrator wiring. Mitigation: existing test coverage for prose/table Replace operations validates behavioral equivalence.

[MergeCandidatesOutput contract change] → Changing from `Vec<String>` to `Vec<Operation>` is a breaking contract change. Mitigation: Phase 5.5 already proved the `Operation` type is serde-compatible. The artifact format changes but behavior is preserved for Replace-only runs.

[Reverse-order edge cases] → Complex multi-op scenarios (insert + remove at same parent, nested operations) need thorough testing. Mitigation: dedicated test suite for ordering with edge case coverage from the explore session analysis.

[Scope anchor deletion] → Deleting a heading used as a scope selector could break future scope resolution. Mitigation: `check_operation_legality` blocks Remove on scope anchor headings (D-018).

[Post-mutation validation coverage] → Structural rule checks may not catch all ADF schema violations. Mitigation: Confluence's publish endpoint rejects invalid ADF; we catch the common cases pre-publish.
