## Why

Phases 6-8 established block-level insert/edit/delete, structural composition, and multi-page orchestration. Three promoted ideas remain unimplemented: table topology changes (row/column add/remove), structural block attribute editing (panel/macro/media attributes), and MCP server integration. Phase 9 delivers all three as the final Structural phase, completing the roadmap vision of full Confluence content control.

## What Changes

### Feature Group A: Table Topology

- `BlockOp` gains 4 new variants: `InsertRow { table_path, index, cells }`, `RemoveRow { table_path, index }`, `InsertColumn { table_path, index }`, `RemoveColumn { table_path, index }`.
- Row operations compose into existing `Operation::Insert` / `Operation::Remove` (Open/Closed — no new Operation variants for rows). `InsertRow` → single `Operation::Insert` of a tableRow. `RemoveRow` → single `Operation::Remove` of a tableRow.
- Column operations compose into N × `Operation::Insert` / `Operation::Remove` (one per row, same composition pattern as Phase 7's `InsertSection`). `InsertColumn` → N inserts of tableCell/tableHeader across all rows.
- **BREAKING**: `check_table_shape_integrity` in verify becomes op-type-aware (Decompose Conditional, Rule of Three). Declared row/column BlockOp operations are allowed; undeclared table mutations remain blocked.
- New builders: `build_table_row`, `build_table_cell`, `build_table_header` composing existing atomic builders.
- Type policy: `tableRow` added to insertable/removable types.
- Structural validity: table column count consistency check after row/column operations.

### Feature Group B: Structural Block Attributes

- `Operation::UpdateAttrs { target_path, attrs }` — new Operation variant. Merges provided attrs into the target node's existing attrs object without touching content.
- `BlockOp::UpdateAttrs { target_path, attrs }` — new BlockOp variant. Translates 1:1 to `Operation::UpdateAttrs`.
- **BREAKING**: Locked structural policy relaxed for `UpdateAttrs` on allowed node types (Rule of Three — 3rd use of locked boundary check). `check_locked_boundary` becomes truly op-type-aware: Replace on locked → blocked; UpdateAttrs on panel/expand/media → allowed; Insert/Remove of children inside locked containers → allowed.
- Type policy: `is_attr_editable_type()` function for panel, expand, mediaSingle.
- Verify: `check_attr_update_legality()` validates target is attr-editable and attrs are valid.
- `apply_update_attrs()` in patch.rs merges attrs.

### Feature Group C: MCP Server

- New crate `atlassy-mcp` in the workspace — MCP server exposing pipeline operations as tools.
- MCP tools: `run` (single page), `run-multi-page` (coordinated), `create-subpage`, `run-readiness`.
- Shared payload contracts via `atlassy-contracts` — no CLI/MCP divergence.
- stdio transport (MCP standard).
- Zero changes to existing crates — purely additive new crate (Facade Pattern).

## Capabilities

### New Capabilities

- `table-row-operations`: InsertRow and RemoveRow BlockOp variants with translate functions composing existing Insert/Remove operations. Row builders for tableRow/tableCell/tableHeader.
- `table-column-operations`: InsertColumn and RemoveColumn BlockOp variants with translate functions composing N Insert/Remove operations across all rows. Header-aware cell generation.
- `structural-attr-editing`: UpdateAttrs Operation and BlockOp variants for editing attributes of locked structural nodes (panel, expand, media) without touching content.
- `mcp-server-integration`: New atlassy-mcp crate exposing pipeline as MCP tools with stdio transport.
- `locked-structural-relaxation`: Op-type-aware locked boundary checking — UpdateAttrs allowed on attr-editable types, child insert/remove inside containers allowed.

### Modified Capabilities

- `operation-type-system`: Operation enum gains UpdateAttrs variant. BlockOp gains InsertRow, RemoveRow, InsertColumn, RemoveColumn, UpdateAttrs variants.
- `block-ops-pipeline-state`: 5 new translate functions added.
- `block-type-policy`: New type policy functions (is_attr_editable_type, is_table_row_insertable). tableRow added to insertable/removable types.
- `table-shape-change-guards`: Decompose Conditional — op-type-aware relaxation for declared row/column operations.
- `structural-validity-check`: Table column count consistency check after table topology operations.
- `operation-ordering`: Ordering handles table-internal operations (same reverse-doc-order, deeper paths first).
- `typed-error-codes`: 4 new error codes.
- `pipeline-state-orchestration`: Pipeline crate workspace gains atlassy-mcp crate.

## Impact

- **atlassy-contracts**: Operation gains 1 variant, BlockOp gains 5 variants, 4 new error codes.
- **atlassy-adf**: builders.rs gains 3 table builders, type_policy.rs gains new functions and types, structural_validity.rs gains table checks, patch.rs gains apply_update_attrs.
- **atlassy-pipeline**: adf_block_ops.rs gains 5 translate functions, verify.rs gains attr check + table guard relaxation + locked boundary refactoring.
- **atlassy-mcp**: New crate (Facade over pipeline).
- **atlassy-cli**: BlockOp construction sites updated for new variants.
- **atlassy-confluence**: No changes.
- **Cargo.toml (workspace)**: New member atlassy-mcp.
