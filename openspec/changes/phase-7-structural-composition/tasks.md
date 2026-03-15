## 1. Type Policy Extraction (refactor, zero behavior change)

- [x] 1.1 Create `atlassy-adf/src/type_policy.rs` with `is_editable_prose()`, `is_insertable_type()`, `is_removable_type()` functions using existing `EDITABLE_PROSE_TYPES` constant
- [x] 1.2 Add `mod type_policy` and public re-exports to `atlassy-adf/src/lib.rs`
- [x] 1.3 Replace `EDITABLE_PROSE_TYPES.contains()` in `adf_block_ops.rs` with `is_insertable_type()` / `is_removable_type()`
- [x] 1.4 Replace `EDITABLE_PROSE_TYPES.contains()` in `verify.rs` `check_operation_legality` with type policy functions
- [x] 1.5 Replace `EDITABLE_PROSE_TYPES.contains()` in `table_guard.rs` with `is_editable_prose()`
- [x] 1.6 Verify no `EDITABLE_PROSE_TYPES.contains()` calls remain outside `type_policy.rs`
- [x] 1.7 Run `cargo test --workspace` — all tests pass with zero behavior change

## 2. ADF Builders Module (new code)

- [x] 2.1 Create `atlassy-adf/src/builders.rs` with `build_text(text: &str) -> Value`
- [x] 2.2 Add `build_paragraph(text: &str) -> Value` using `build_text`
- [x] 2.3 Add `build_heading(level: u8, text: &str) -> Result<Value, AdfError>` with level 1-6 validation
- [x] 2.4 Add `build_table(rows: usize, cols: usize, header_row: bool) -> Result<Value, AdfError>` with zero-dimension validation
- [x] 2.5 Add `build_list(ordered: bool, items: &[&str]) -> Result<Value, AdfError>` with empty-items validation
- [x] 2.6 Add `build_section(level: u8, heading_text: &str, body_blocks: &[Value]) -> Result<Vec<Value>, AdfError>`
- [x] 2.7 Add `mod builders` and public re-exports to `atlassy-adf/src/lib.rs`

## 3. Section Boundary Detection Module (new code)

- [x] 3.1 Create `atlassy-adf/src/section.rs` with `SectionRange` struct
- [x] 3.2 Implement `find_section_range(adf: &Value, heading_path: &str) -> Result<SectionRange, AdfError>`
- [x] 3.3 Implement heading level extraction from `attrs.level`
- [x] 3.4 Implement sibling walk: forward from heading until same-or-higher-level heading or end of array
- [x] 3.5 Implement `split_parent_index` utility if not already available from Phase 6
- [x] 3.6 Add `mod section` and public re-exports to `atlassy-adf/src/lib.rs`

## 4. BlockOp New Variants (atlassy-contracts)

- [x] 4.1 Add `InsertSection { parent_path: String, index: usize, heading_level: u8, heading_text: String, body_blocks: Vec<Value> }` to `BlockOp` enum
- [x] 4.2 Add `RemoveSection { heading_path: String }` to `BlockOp` enum
- [x] 4.3 Add `InsertTable { parent_path: String, index: usize, rows: usize, cols: usize, header_row: bool }` to `BlockOp` enum
- [x] 4.4 Add `InsertList { parent_path: String, index: usize, ordered: bool, items: Vec<String> }` to `BlockOp` enum
- [x] 4.5 Verify serde round-trip for all new variants

## 5. Error Codes (atlassy-contracts)

- [x] 5.1 Add `SectionBoundaryInvalid` variant to `ErrorCode` with `as_str` returning `"ERR_SECTION_BOUNDARY_INVALID"`
- [x] 5.2 Add `StructuralCompositionFailed` variant to `ErrorCode` with `as_str` returning `"ERR_STRUCTURAL_COMPOSITION_FAILED"`
- [x] 5.3 Update `ErrorCode::ALL` array and test coverage

## 6. ADF Error Variants (atlassy-adf)

- [x] 6.1 Add `SectionBoundaryInvalid(String)` variant to `AdfError`
- [x] 6.2 Add `StructuralCompositionFailed(String)` variant to `AdfError`

## 7. Translate Functions (atlassy-pipeline/states/adf_block_ops.rs)

- [x] 7.1 Extract existing `BlockOp::Insert` handling into `translate_insert()` function
- [x] 7.2 Extract existing `BlockOp::Remove` handling into `translate_remove()` function
- [x] 7.3 Implement `translate_insert_section()` — uses `build_heading` + body_blocks to construct N Operation::Insert commands at consecutive indices
- [x] 7.4 Implement `translate_remove_section()` — uses `find_section_range()` with `scoped_adf` to detect boundary, produces N Operation::Remove in reverse order
- [x] 7.5 Implement `translate_insert_table()` — uses `build_table()` to construct ADF, produces 1 Operation::Insert
- [x] 7.6 Implement `translate_insert_list()` — uses `build_list()` to construct ADF, produces 1 Operation::Insert
- [x] 7.7 Update `translate_block_op` match to dispatch all 6 variants to their translate functions
- [x] 7.8 Add `scoped_adf: &Value` parameter to `translate_block_op` signature (passed from FetchOutput)

## 8. Type Allowlist Expansion

- [x] 8.1 Add `"table"` to `INSERTABLE_BLOCK_TYPES` in `type_policy.rs`
- [x] 8.2 Add `"table"` to `REMOVABLE_BLOCK_TYPES` in `type_policy.rs`
- [x] 8.3 Verify `is_insertable_type("table")` returns `true`

## 9. Locked Boundary Check Extraction (merge/verify)

- [x] 9.1 Extract `check_locked_boundary(operation: &Operation, locked_paths: &[&str]) -> Option<PipelineError>` function
- [x] 9.2 Update `merge_candidates.rs` to call `check_locked_boundary` instead of inline locked check
- [x] 9.3 Update `verify.rs` to call `check_locked_boundary` where applicable
- [x] 9.4 Verify locked boundary behavior is identical to pre-extraction

## 10. Orchestrator Wiring

- [x] 10.1 Pass `scoped_adf` from `FetchOutput` to `run_adf_block_ops_state`
- [x] 10.2 Verify all 6 BlockOp variants flow through merge → sort → apply → verify correctly

## 11. Unit Tests (atlassy-adf)

- [x] 11.1 Test `build_text`, `build_paragraph`, `build_heading` — valid construction and heading level validation
- [x] 11.2 Test `build_table` — valid dimensions, header/no-header, zero-dimension rejection
- [x] 11.3 Test `build_list` — ordered/unordered, empty items rejection
- [x] 11.4 Test `build_section` — heading + body, empty body, invalid heading level
- [x] 11.5 Test `find_section_range` — normal section, end-of-doc section, empty section, non-heading target, out-of-bounds
- [x] 11.6 Test `is_editable_prose`, `is_insertable_type`, `is_removable_type` — all type categories
- [x] 11.7 Test builder outputs pass `check_structural_validity`

## 12. Integration Tests (atlassy-pipeline)

- [x] 12.1 Test InsertSection: heading + 2 paragraphs inserted at correct positions
- [x] 12.2 Test RemoveSection: heading + body blocks removed, adjacent content preserved
- [x] 12.3 Test InsertTable: valid table structure inserted and publishes
- [x] 12.4 Test InsertList: valid list structure inserted and publishes
- [x] 12.5 Test mixed run: InsertSection + Replace text in same batch
- [x] 12.6 Test backward compatibility: Replace-only and Phase 6 Insert/Remove runs unchanged
- [x] 12.7 Test error cases: out-of-scope section insert, non-heading RemoveSection target

## 13. Contract + Serialization Tests

- [x] 13.1 Test all new BlockOp variants serde round-trip
- [x] 13.2 Test existing BlockOp variants unchanged after addition

## 14. Final Validation

- [x] 14.1 Run `cargo test --workspace` — all tests pass
- [x] 14.2 Run `cargo clippy --workspace` — zero warnings
- [x] 14.3 Verify no `EDITABLE_PROSE_TYPES.contains()` outside `type_policy.rs`
- [x] 14.4 Verify `translate_block_op` dispatches all 6 variants to extracted functions
