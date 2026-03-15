## Why

Phase 6 provides generic Insert/Remove primitives that work at the individual block level. Users need higher-level structural operations — inserting sections (heading + body), creating tables and lists, and removing entire sections — without manually constructing ADF JSON or calculating block boundaries. Phase 7 composes Phase 6's primitives into these higher-level operations, following Builder and Strategy patterns. This also addresses a Rule-of-Three Shotgun Surgery smell: `EDITABLE_PROSE_TYPES` is checked in 6 files / 14 locations, and expanding the type allowlist requires Shotgun Surgery without refactoring first.

## What Changes

- **BREAKING**: `BlockOp` enum gains 4 new variants: `InsertSection { parent_path, index, heading_level, heading_text, body_blocks }`, `RemoveSection { heading_path }`, `InsertTable { parent_path, index, rows, cols, header_row }`, `InsertList { parent_path, index, ordered, items }`.
- New `atlassy-adf/src/builders.rs` module — atomic + composite ADF node construction functions (Builder Pattern): `build_text`, `build_paragraph`, `build_heading`, `build_table`, `build_list`, `build_section`.
- New `atlassy-adf/src/section.rs` module — `find_section_range()` for detecting section boundaries in ADF (Extract Class): walks sibling blocks from a heading to the next same-or-higher-level heading.
- New `atlassy-adf/src/type_policy.rs` module — replaces scattered `EDITABLE_PROSE_TYPES` checks with query functions `is_insertable_type()`, `is_removable_type()`, `is_editable_prose()` (Rule of Three / Replace Magic Number with Symbolic Constant).
- `table` added to insertable and removable block types via type policy functions.
- `adf_block_ops.rs` gains 4 new translate functions (Strategy Pattern / Extract Method): `translate_insert_section()`, `translate_remove_section()`, `translate_insert_table()`, `translate_insert_list()`. `RemoveSection` requires access to `scoped_adf` for section boundary detection.
- Extract `check_locked_boundary()` function from merge/verify locked structural checks for future Phase 8-9 extensibility (Decompose Conditional).

## Capabilities

### New Capabilities

- `adf-node-builders`: ADF node construction functions using Builder Pattern — atomic builders (text, paragraph, heading) and composite builders (table, list, section) that compose atomics. Designed for Phase 9 extension with table-internal builders.
- `section-boundary-detection`: Domain logic for finding section ranges in ADF — given a heading path, determines all blocks belonging to that section (heading through body blocks until next same-or-higher-level heading).
- `block-type-policy`: Query functions replacing scattered `EDITABLE_PROSE_TYPES` constant checks. Centralizes insertable/removable type decisions in one module. Designed for Phase 9 extension with table-internal type policies.
- `section-operations`: InsertSection and RemoveSection BlockOp variants with their translate functions. InsertSection uses builders to construct heading + body, producing N Operation::Insert commands. RemoveSection uses section boundary detection to find the range, producing N Operation::Remove commands in reverse order.
- `table-list-creation`: InsertTable and InsertList BlockOp variants with their translate functions. InsertTable uses builders to construct a complete table ADF node. InsertList constructs bulletList or orderedList ADF.

### Modified Capabilities

- `operation-type-system`: BlockOp enum gains InsertSection, RemoveSection, InsertTable, InsertList variants.
- `block-ops-pipeline-state`: 4 new translate functions added, `translate_block_op` gains `scoped_adf` parameter for section boundary detection.
- `table-shape-change-guards`: Extract `check_locked_boundary()` function from locked structural check logic for op-type-aware enforcement and future extensibility.
- `typed-error-codes`: New error codes for section boundary failures and table/list construction failures.

## Impact

- **atlassy-contracts**: `BlockOp` gains 4 variants. New error codes added.
- **atlassy-adf**: 3 new modules (`builders.rs`, `section.rs`, `type_policy.rs`). `lib.rs` updated with new exports. `EDITABLE_PROSE_TYPES` references moved to type_policy functions.
- **atlassy-pipeline**: `adf_block_ops.rs` gains 4 translate functions + `scoped_adf` parameter. `merge_candidates.rs` and `verify.rs` use type policy functions instead of direct constant checks. Locked boundary check extracted.
- **atlassy-cli**: `BlockOp` construction sites updated for new enum variants.
- **atlassy-confluence**: No changes.
