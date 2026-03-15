## Context

Phase 6 added generic Insert/Remove operations. Phase 7 composed them into sections, tables, and lists via BlockOp expansion (Strategy Pattern). Phase 8 added multi-page orchestration. The pipeline architecture is stable: merge collects operations, patch applies them, verify validates results.

Three promoted ideas remain: table topology (from `ideas/2026-03-advanced-table-editing-modes.md`), structural block attributes (from `ideas/2026-03-structural-block-editing-support.md`), and MCP server (from `ideas/2026-03-mcp-server-integration.md`).

These three features have zero overlap in implementation but share the `Operation` enum extension point. The design unifies them in one phase with clear feature group boundaries and a preparatory refactoring step.

## Goals / Non-Goals

**Goals:**

- Table row and column add/remove with strict bounds checking (Mode 2 from the ideas doc).
- Structural block attribute editing for panel, expand, and media nodes.
- MCP server exposing the full pipeline as tools for AI agents.
- Locked structural policy relaxation to allow UpdateAttrs and container child operations (Rule of Three — 3rd concrete use case).

**Non-Goals:**

- Table merge/split cells (Mode 3 from ideas doc — deferred, too complex for Phase 9).
- Table attribute changes (table-level attrs like width — deferred).
- Layout-aware transformations (layoutSection/layoutColumn editing — deferred).
- Extension/macro parameter editing beyond simple attrs (schema complexity — deferred).
- MCP auth beyond env-var-based API tokens (existing pattern sufficient).
- Parallel multi-page execution (Phase 8 is sequential, stays sequential).

## Decisions

### D1: Row operations compose existing Insert/Remove (Open/Closed)

`InsertRow` and `RemoveRow` use the same `Operation::Insert` / `Operation::Remove` primitives from Phase 6. A tableRow is a child of the table's content array — inserting/removing it follows the same semantics as inserting/removing a paragraph from doc.content.

`BlockOp::InsertRow { table_path, index, cells }` → `translate_insert_row()` → `Operation::Insert { parent_path: "{table_path}/content", index, block: tableRow }`.

No new Operation variant needed. `tableRow` is added to `INSERTABLE_BLOCK_TYPES` / `REMOVABLE_BLOCK_TYPES` via type policy.

Alternative considered: `Operation::InsertRow` as a new variant. Rejected because the semantics are identical to generic Insert — it's inserting a child into an array. The table-specific validation (cell count must match other rows) belongs in translate and verify, not in the Operation type.

### D2: Column operations compose N Insert/Remove across rows (Strategy Pattern)

Column operations are the multi-row equivalent of Phase 7's section operations. `InsertColumn` adds a cell to every row at the same column index. `RemoveColumn` removes a cell from every row.

`BlockOp::InsertColumn { table_path, index }` → `translate_insert_column()`:
1. Read the table's content array to find all rows.
2. For the header row (if present): generate `Operation::Insert` with `tableHeader` cell.
3. For each data row: generate `Operation::Insert` with `tableCell` cell.
4. Return N Insert operations (one per row).

This follows the same composition pattern as `InsertSection` (Phase 7): high-level BlockOp → N low-level Operations. The reverse-document-order sorting from Phase 6 handles the multi-op batch correctly.

Alternative considered: `Operation::InsertColumn` as a single variant that knows about all rows. Rejected because the pipeline's sort/apply/verify chain already handles multi-op batches. A single variant would bypass the established flow.

### D3: UpdateAttrs is a new Operation variant (different semantics)

`Operation::UpdateAttrs { target_path, attrs }` is fundamentally different from Replace, Insert, and Remove:
- Replace changes a leaf value.
- Insert adds a new node.
- Remove deletes a node.
- UpdateAttrs modifies the `attrs` object of an existing node without touching its content.

The `apply_update_attrs` function navigates to `target_path`, accesses the `attrs` field, and merges the provided attrs (overwriting keys that exist, adding keys that don't). It never touches `content`.

This IS a new Operation variant because the semantics don't map to any existing variant. Following Open/Closed: the variant is additive, existing code handles it via a new match arm.

### D4: Locked structural relaxation — Rule of Three (Decompose Conditional)

The locked boundary check has been considered three times:
1. Phase 7: deferred (first use, no concrete need).
2. Phase 8: deferred (multi-page, no per-page capability change).
3. Phase 9: `UpdateAttrs` on panel/expand/media IS the concrete use case.

Rule of Three mandates refactoring now. `check_locked_boundary` becomes truly op-type-aware:

- `Replace` on locked node → BLOCKED (existing behavior, unchanged).
- `UpdateAttrs` on attr-editable locked type (panel, expand, mediaSingle) → ALLOWED.
- `UpdateAttrs` on non-attr-editable locked type → BLOCKED.
- `Insert` child inside locked container → ALLOWED (container child insertion).
- `Remove` child inside locked container → ALLOWED.
- `Remove` the locked container itself → BLOCKED.

This is Decompose Conditional: replacing a boolean `is_locked → block` with a nuanced decision tree based on operation type and target.

### D5: Table shape guard becomes op-type-aware

`check_table_shape_integrity` currently blocks ALL table structural changes. Phase 9 relaxes this for declared row/column operations:

- Row add/remove from a declared `BlockOp::InsertRow`/`RemoveRow` → ALLOWED.
- Column add/remove from a declared `BlockOp::InsertColumn`/`RemoveColumn` → ALLOWED.
- Undeclared table structural mutations → BLOCKED (existing behavior).
- Cell merge/split → BLOCKED (Mode 3, not in Phase 9 scope).
- Table attribute changes → BLOCKED (not in Phase 9 scope).

The guard receives the operation manifest (list of declared BlockOp operations) to distinguish declared from undeclared mutations.

### D6: MCP server as Facade (new crate, zero existing changes)

`atlassy-mcp` is a new workspace member that depends on `atlassy-pipeline`, `atlassy-contracts`, and `atlassy-confluence`. It exposes MCP tools that map to existing pipeline capabilities:

- `atlassy_run` → `Orchestrator::run()` with a `RunRequest`.
- `atlassy_run_multi_page` → `MultiPageOrchestrator::run()` with a `MultiPageRequest`.
- `atlassy_create_subpage` → `ConfluenceClient::create_page()`.
- `atlassy_readiness` → readiness check from existing CLI logic.

The MCP server uses stdio transport (MCP standard). Tool schemas are derived from the contract types in `atlassy-contracts`. No existing crate code changes — the MCP crate is a pure consumer.

### D7: Preparatory refactoring as Step 1 (zero behavior change)

Before adding features, refactor:
1. Extract table shape guard into op-type-aware function (Decompose Conditional).
2. Add type policy functions: `is_attr_editable_type()`, `is_table_row_insertable()`.
3. Add table builders: `build_table_row`, `build_table_cell`, `build_table_header`.
4. Expand structural validity for table column consistency.

All zero behavior change. All existing tests pass. Then add features.

## Risks / Trade-offs

[Column operation ordering complexity] → Column inserts across N rows must not interleave with other operations. Mitigation: existing reverse-doc-order sorting processes deeper paths first — column ops within a table are deeper than doc-level ops, so they naturally group.

[UpdateAttrs partial merge semantics] → Merging attrs could overwrite critical fields if the caller provides wrong keys. Mitigation: `check_attr_update_legality` validates allowed attr keys per node type (e.g., panel only allows `panelType`; media only allows `alt`, `title`).

[MCP crate dependency management] → MCP SDK choice affects build complexity. Mitigation: start with minimal hand-rolled MCP protocol over stdio. Upgrade to SDK if maintenance burden justifies it.

[Table column consistency after operations] → After column add/remove, all rows must have the same number of cells. Mitigation: structural validity check validates column count consistency post-mutation.

[Locked relaxation scope creep] → Allowing UpdateAttrs on panels could tempt expanding to all locked types. Mitigation: explicit `is_attr_editable_type()` allowlist controls which types accept UpdateAttrs. Expansion requires a policy decision.
