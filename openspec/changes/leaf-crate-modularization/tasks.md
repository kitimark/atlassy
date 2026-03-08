## 1. Modularize atlassy-contracts

- [ ] 1.1 Create `crates/atlassy-contracts/src/constants.rs` with all 21 `pub const` declarations (lines 6-30)
- [ ] 1.2 Create `crates/atlassy-contracts/src/types.rs` with all structs, enums, and impl blocks (lines 32-421)
- [ ] 1.3 Create `crates/atlassy-contracts/src/validation.rs` with all validation functions and private helpers (lines 423-660)
- [ ] 1.4 Replace `crates/atlassy-contracts/src/lib.rs` with facade: `mod` declarations + `pub use *` re-exports
- [ ] 1.5 Run `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` — all pass

## 2. Modularize atlassy-confluence

- [ ] 2.1 Create `crates/atlassy-confluence/src/types.rs` with `FetchPageResponse`, `PublishPageResponse`, `CreatePageResponse`, `ConfluenceError`, and `ConfluenceClient` trait
- [ ] 2.2 Create `crates/atlassy-confluence/src/stub.rs` with `StubPage`, `StubConfluenceClient`, inherent impls, and `ConfluenceClient` trait impl
- [ ] 2.3 Create `crates/atlassy-confluence/src/live.rs` with `LiveConfluenceClient`, inherent impl, `ConfluenceClient` trait impl, and private payload builders
- [ ] 2.4 Move 4 payload-builder tests from `src/tests.rs` into `#[cfg(test)] mod tests` block in `src/live.rs`
- [ ] 2.5 Update `src/tests.rs` to contain only the 3 remaining stub tests
- [ ] 2.6 Replace `crates/atlassy-confluence/src/lib.rs` with facade: `mod` declarations + `pub use *` re-exports + `#[cfg(test)] mod tests;`
- [ ] 2.7 Run `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` — all pass

## 3. Modularize atlassy-adf

- [ ] 3.1 Create `crates/atlassy-adf/src/path.rs` with `document_order_sort`, `compare_path_segments`, `is_within_allowed_scope`, `is_path_within_or_descendant`, `canonicalize_mapped_path`, `is_json_pointer`, `escape_pointer_segment`, `parent_path` — adjust visibility of previously-private functions to `pub(crate)` where needed
- [ ] 3.2 Create `crates/atlassy-adf/src/index.rs` with `build_node_path_index`, `build_node_path_index_inner`, `path_has_ancestor_type`, `collect_text` — import path utilities via `use crate::path::*`
- [ ] 3.3 Create `crates/atlassy-adf/src/scope.rs` with `resolve_scope`, `full_page_resolution`, `find_heading_paths`, `find_block_paths`, `expand_heading_to_section`, `heading_level` — import from `crate::path` and `crate::index`
- [ ] 3.4 Create `crates/atlassy-adf/src/patch.rs` with `normalize_changed_paths`, `build_patch_ops`, `apply_patch_ops`, `ensure_paths_in_scope` — import from `crate::path`
- [ ] 3.5 Create `crates/atlassy-adf/src/table_guard.rs` with `discover_target_path`, `is_table_cell_text_path`, `is_table_shape_or_attr_path`, `markdown_for_path` — import from `crate::path` and `crate::index`
- [ ] 3.6 Create `crates/atlassy-adf/src/bootstrap.rs` with `is_page_effectively_empty`, `bootstrap_scaffold` — zero intra-crate imports
- [ ] 3.7 Replace `crates/atlassy-adf/src/lib.rs` with facade: cross-cutting types and constants + `mod` declarations + `pub use *` re-exports
- [ ] 3.8 Run `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` — all pass

## 4. Final verification

- [ ] 4.1 Verify no downstream import changes in `atlassy-pipeline` or `atlassy-cli` — grep for changed import paths
- [ ] 4.2 Verify test count is preserved at 107: `grep -r '#\[test\]' crates/ --include='*.rs' | wc -l`
- [ ] 4.3 Verify no inline test blocks in entry files: `grep -rn 'mod tests {' crates/*/src/lib.rs crates/*/src/main.rs` returns zero matches
