## 1. Preparatory Refactoring (zero behavior change)

- [x] 1.1 Add `is_attr_editable_type()` function to `type_policy.rs` — returns true for panel, expand, mediaSingle
- [x] 1.2 Add `tableRow` to `INSERTABLE_BLOCK_TYPES` and `REMOVABLE_BLOCK_TYPES` in `type_policy.rs`
- [x] 1.3 Add `build_table_row(cells: &[Value]) -> Value` builder function in `builders.rs`
- [x] 1.4 Add `build_table_cell(content: &str) -> Value` builder function in `builders.rs`
- [x] 1.5 Add `build_table_header(content: &str) -> Value` builder function in `builders.rs`
- [x] 1.6 Add table column consistency check to `structural_validity.rs` — validate all rows have same cell count
- [x] 1.7 Refactor `check_table_shape_integrity` to accept operation manifest for op-type-aware decisions (Decompose Conditional)
- [x] 1.8 Run `cargo test --workspace` — all existing tests pass with zero behavior change

## 2. Operation Enum + BlockOp Variants (atlassy-contracts)

- [x] 2.1 Add `Operation::UpdateAttrs { target_path: String, attrs: Value }` variant
- [x] 2.2 Add `BlockOp::InsertRow { table_path: String, index: usize, cells: Vec<String> }` variant
- [x] 2.3 Add `BlockOp::RemoveRow { table_path: String, index: usize }` variant
- [x] 2.4 Add `BlockOp::InsertColumn { table_path: String, index: usize }` variant
- [x] 2.5 Add `BlockOp::RemoveColumn { table_path: String, index: usize }` variant
- [x] 2.6 Add `BlockOp::UpdateAttrs { target_path: String, attrs: Value }` variant
- [x] 2.7 Verify serde round-trip for all new variants

## 3. Error Codes (atlassy-contracts)

- [x] 3.1 Add `TableRowInvalid` variant to `ErrorCode` — `"ERR_TABLE_ROW_INVALID"`
- [x] 3.2 Add `TableColumnInvalid` variant — `"ERR_TABLE_COLUMN_INVALID"`
- [x] 3.3 Add `AttrUpdateBlocked` variant — `"ERR_ATTR_UPDATE_BLOCKED"`
- [x] 3.4 Add `AttrSchemaViolation` variant — `"ERR_ATTR_SCHEMA_VIOLATION"`
- [x] 3.5 Update `ErrorCode::ALL` array and test coverage

## 4. ADF Error Variants (atlassy-adf)

- [x] 4.1 Add `TableRowInvalid(String)` variant to `AdfError`
- [x] 4.2 Add `TableColumnInvalid(String)` variant to `AdfError`
- [x] 4.3 Add `AttrUpdateBlocked(String)` variant to `AdfError`
- [x] 4.4 Add `AttrSchemaViolation(String)` variant to `AdfError`

## 5. apply_update_attrs (atlassy-adf/src/patch.rs)

- [x] 5.1 Add `Operation::UpdateAttrs` arm to `apply_operations` match — delegates to `apply_update_attrs`
- [x] 5.2 Implement `apply_update_attrs(candidate, target_path, attrs)` — navigate to target, merge attrs into existing attrs object
- [x] 5.3 Add `Operation::UpdateAttrs` arm to `validate_operations` — check path in scope, not root
- [x] 5.4 Handle case where target has no existing attrs — create attrs object

## 6. Ordering Updates (atlassy-adf/src/ordering.rs)

- [x] 6.1 Add `UpdateAttrs` handling to `extract_path_info` — treat as leaf operation (like Replace)
- [x] 6.2 Ensure UpdateAttrs is partitioned with replaces (before structural ops)
- [x] 6.3 Verify table-internal operations sort correctly (deeper paths first via existing logic)

## 7. Table Row Translate Functions (atlassy-pipeline/states/adf_block_ops.rs)

- [x] 7.1 Implement `translate_insert_row(table_path, index, cells, allowed_scope_paths, scoped_adf)` — validate cell count matches existing columns, build tableRow, return single Operation::Insert
- [x] 7.2 Implement `translate_remove_row(table_path, index, allowed_scope_paths, scoped_adf)` — validate index in bounds, not last row, return single Operation::Remove
- [x] 7.3 Add `InsertRow` and `RemoveRow` arms to `translate_block_op` match

## 8. Table Column Translate Functions (atlassy-pipeline/states/adf_block_ops.rs)

- [x] 8.1 Implement `translate_insert_column(table_path, index, allowed_scope_paths, scoped_adf)` — read table rows, generate N Operation::Insert (tableHeader for header row, tableCell for data rows), validate index in bounds
- [x] 8.2 Implement `translate_remove_column(table_path, index, allowed_scope_paths, scoped_adf)` — read table rows, generate N Operation::Remove (one per row), validate index in bounds, not last column
- [x] 8.3 Add `InsertColumn` and `RemoveColumn` arms to `translate_block_op` match

## 9. UpdateAttrs Translate Function (atlassy-pipeline/states/adf_block_ops.rs)

- [x] 9.1 Implement `translate_update_attrs(target_path, attrs, allowed_scope_paths, scoped_adf)` — validate scope, validate target is attr-editable, return single Operation::UpdateAttrs
- [x] 9.2 Add `UpdateAttrs` arm to `translate_block_op` match

## 10. Verify Updates (atlassy-pipeline/states/verify.rs)

- [x] 10.1 Add `UpdateAttrs` handling to `check_operation_legality` — validate target is attr-editable, attrs are allowed keys for node type
- [x] 10.2 Refactor `check_locked_boundary` to be op-type-aware (Rule of Three): Replace on locked → blocked; UpdateAttrs on attr-editable → allowed; Insert/Remove child inside container → allowed; Remove container itself → blocked
- [x] 10.3 Update `check_table_shape_integrity` to accept operation manifest and allow declared row/column operations
- [x] 10.4 Add allowed-attrs-per-node-type validation (panel: panelType; expand: title; mediaSingle: alt, title, width, height)

## 11. MCP Server Crate (atlassy-mcp)

- [x] 11.1 Create `crates/atlassy-mcp/Cargo.toml` with dependencies on atlassy-pipeline, atlassy-contracts, atlassy-confluence
- [x] 11.2 Add `atlassy-mcp` to workspace members in root `Cargo.toml`
- [x] 11.3 Create `crates/atlassy-mcp/src/main.rs` — MCP server entry point with stdio transport
- [x] 11.4 Implement MCP protocol handling: `initialize`, `tools/list`, `tools/call` message types
- [x] 11.5 Implement `atlassy_run` tool — constructs RunRequest, calls Orchestrator::run(), returns RunSummary
- [x] 11.6 Implement `atlassy_run_multi_page` tool — constructs MultiPageRequest, calls MultiPageOrchestrator::run(), returns MultiPageSummary
- [x] 11.7 Implement `atlassy_create_subpage` tool — calls ConfluenceClient::create_page()
- [x] 11.8 Implement env-var credential loading (same pattern as CLI)
- [x] 11.9 Define JSON schemas for tool inputs derived from contract types

## 12. CLI Updates (atlassy-cli)

- [x] 12.1 Update `BlockOp` construction sites in `run.rs` for 5 new variants
- [x] 12.2 Update `BlockOp` construction sites in `run_batch.rs` for new variants

## 13. Unit Tests (atlassy-adf)

- [x] 13.1 Test `build_table_row`, `build_table_cell`, `build_table_header` — valid construction
- [x] 13.2 Test `apply_update_attrs` — merge attrs, create attrs, target not found
- [x] 13.3 Test `structural_validity` table column consistency — consistent, inconsistent, empty table
- [x] 13.4 Test ordering with UpdateAttrs — treated as leaf, sorted with replaces
- [x] 13.5 Test `is_attr_editable_type` — panel/expand/mediaSingle true, others false

## 14. Integration Tests (atlassy-pipeline)

- [x] 14.1 Test InsertRow: row added with correct cell count
- [x] 14.2 Test RemoveRow: row removed, remaining rows intact
- [x] 14.3 Test InsertColumn: cell added to every row at correct index
- [x] 14.4 Test RemoveColumn: cell removed from every row
- [x] 14.5 Test UpdateAttrs: panel panelType changed, content unchanged
- [x] 14.6 Test UpdateAttrs on non-attr-editable type: rejected
- [x] 14.7 Test table shape guard: declared row/column op allowed, undeclared blocked
- [x] 14.8 Test locked boundary: UpdateAttrs on panel allowed, Replace on panel blocked
- [x] 14.9 Test backward compatibility: all Phase 6-8 operations unchanged

## 15. MCP Server Tests

- [x] 15.1 Test tools/list returns expected tool set
- [x] 15.2 Test atlassy_run tool with stub backend
- [x] 15.3 Test atlassy_create_subpage tool with stub backend
- [x] 15.4 Test tool call with invalid input returns error

## 16. Final Validation

- [x] 16.1 Run `cargo test --workspace` — all tests pass
- [x] 16.2 Run `cargo clippy --workspace` — zero warnings
- [x] 16.3 Verify `atlassy-mcp` builds and responds to tools/list
- [x] 16.4 Verify no existing pipeline state files were modified (only verify.rs updated)
- [x] 16.5 Verify table shape guard allows declared ops and blocks undeclared ops
