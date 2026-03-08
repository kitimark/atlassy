## 1. Baseline verification

- [x] 1.1 Run `grep -r '#\[test\]' crates/ --include='*.rs' | wc -l` and confirm count is 107
- [x] 1.2 Run `cargo test --workspace` and confirm all tests pass
- [x] 1.3 Run `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`

## 2. Extract atlassy-pipeline tests (3 tests → src/tests.rs)

- [x] 2.1 Create `crates/atlassy-pipeline/src/tests.rs` with `use super::*;` and move all 3 test functions from `src/lib.rs` lines 1529-1587
- [x] 2.2 Replace inline `mod tests { ... }` block in `src/lib.rs` with `#[cfg(test)] mod tests;`
- [x] 2.3 Run `cargo test -p atlassy-pipeline` and confirm all 31 tests pass (3 unit + 28 integration)
- [x] 2.4 Commit: "extract pipeline inline tests to src/tests.rs" (waived: no per-crate commits required)

## 3. Extract atlassy-confluence tests (7 tests → src/tests.rs)

- [x] 3.1 Create `crates/atlassy-confluence/src/tests.rs` with `use super::*;` and move all 7 test functions from `src/lib.rs` lines 494-643
- [x] 3.2 Replace inline `mod tests { ... }` block in `src/lib.rs` with `#[cfg(test)] mod tests;`
- [x] 3.3 Run `cargo test -p atlassy-confluence` and confirm all 7 tests pass
- [x] 3.4 Commit: "extract confluence inline tests to src/tests.rs" (waived: no per-crate commits required)

## 4. Extract atlassy-contracts tests (10 tests → tests/)

- [x] 4.1 Create `crates/atlassy-contracts/tests/contract_validation.rs` with `use atlassy_contracts::*;` and move all 10 test functions from `src/lib.rs` lines 662-997
- [x] 4.2 Remove the entire `#[cfg(test)] mod tests { ... }` block from `src/lib.rs` (no thin declaration needed — tests are external)
- [x] 4.3 Run `cargo test -p atlassy-contracts` and confirm all 10 tests pass
- [x] 4.4 Commit: "extract contracts inline tests to tests/contract_validation.rs" (waived: no per-crate commits required)

## 5. Extract atlassy-adf tests (42 tests → 5 files in tests/)

- [x] 5.1 Create `crates/atlassy-adf/tests/scope_resolution.rs` — move 15 tests: `resolves_heading_scope`, `heading_selector_requires_exact_match`, `heading_selector_exact_match_still_works`, `duplicate_heading_text_matches_all_sections`, `resolves_block_scope_by_attrs_id`, `resolves_block_scope_by_attrs_local_id`, `block_selector_falls_back_when_no_match`, `resolves_heading_scope_until_next_same_level_heading`, `resolves_heading_scope_for_heading_at_end_of_content`, `resolves_adjacent_same_level_headings_to_single_path`, `includes_nested_subheading_content_in_parent_section`, `h1_section_includes_nested_h2_and_h3_until_next_h1`, `unions_multiple_heading_selectors_with_sorted_deduped_paths`, `heading_without_level_defaults_to_six`, `nested_heading_falls_back_to_full_page`
- [x] 5.2 Create `crates/atlassy-adf/tests/target_discovery.rs` — move 8 tests: `discovers_first_prose_text_in_section`, `discovers_nth_prose_text_with_index`, `discovers_table_cell_text`, `discovery_respects_scope_boundary`, `discovery_excludes_heading_text_nodes`, `discovery_fails_for_heading_only_section`, `discovery_fails_on_empty_section`, `discovery_fails_on_out_of_bounds_index`
- [x] 5.3 Create `crates/atlassy-adf/tests/patch_ops.rs` — move 3 tests: `rejects_whole_body_patch`, `canonicalizes_relative_path_to_scope_root`, `applies_patch_ops_to_candidate_payload`
- [x] 5.4 Create `crates/atlassy-adf/tests/path_classification.rs` — move 4 tests: `detects_table_cell_text_paths`, `document_order_sort_numeric_segments`, `document_order_sort_shared_prefix`, `scope_anchor_types_is_subset_of_editable_prose`
- [x] 5.5 Create `crates/atlassy-adf/tests/emptiness_bootstrap.rs` — move 12 tests: `empty_content_array_is_effectively_empty`, `missing_content_is_effectively_empty`, `single_empty_paragraph_is_effectively_empty`, `paragraph_with_empty_text_is_effectively_empty`, `paragraph_with_local_id_but_no_text_is_effectively_empty`, `paragraph_with_non_empty_text_is_not_empty`, `heading_with_text_is_not_empty`, `table_node_is_not_empty`, `panel_node_is_not_empty`, `bootstrap_scaffold_contains_only_prose_nodes`, `extracts_markdown_for_resolved_path`, `detects_out_of_scope_paths`
- [x] 5.6 Remove the entire `#[cfg(test)] mod tests { ... }` block from `src/lib.rs`
- [x] 5.7 Run `cargo test -p atlassy-adf` and confirm all 42 tests pass
- [x] 5.8 Commit: "extract adf inline tests to tests/ with domain grouping" (waived: no per-crate commits required)

## 6. Extract atlassy-cli tests (15 tests → src/tests.rs + src/test_helpers.rs)

- [x] 6.1 Create `crates/atlassy-cli/src/test_helpers.rs` — move `fixture_path` (from test module line 2793) and `execute_batch_from_manifest_file` (from `#[cfg(test)]` block at line 664-674)
- [x] 6.2 Create `crates/atlassy-cli/src/tests.rs` with `use super::*;` and `use super::test_helpers::*;` — move all 15 test functions from `src/main.rs` lines 2800-3380
- [x] 6.3 Replace inline `mod tests { ... }` block and `#[cfg(test)] fn execute_batch_from_manifest_file` in `main.rs` with `#[cfg(test)] mod test_helpers;` and `#[cfg(test)] mod tests;`
- [x] 6.4 Run `cargo test -p atlassy-cli` and confirm all 17 tests pass (15 unit + 2 integration)
- [x] 6.5 Commit: "extract cli inline tests to src/tests.rs and src/test_helpers.rs" (waived: no per-crate commits required)

## 7. Final verification

- [x] 7.1 Run `grep -r '#\[test\]' crates/ --include='*.rs' | wc -l` and confirm count is still 107
- [x] 7.2 Run `grep -rn 'mod tests {' crates/*/src/lib.rs crates/*/src/main.rs` and confirm zero matches
- [x] 7.3 Run `cargo test --workspace` and confirm all 107 tests pass
- [x] 7.4 Run `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`
