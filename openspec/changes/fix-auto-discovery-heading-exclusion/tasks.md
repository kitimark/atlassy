## 1. Constants and Sort Function

- [ ] 1.1 Add `SCOPE_ANCHOR_TYPES` constant to `crates/atlassy-adf/src/lib.rs` alongside `EDITABLE_PROSE_TYPES` with value `&["heading"]` and doc comment explaining its purpose as a discovery exclusion filter for scope anchor node types
- [ ] 1.2 Add `document_order_sort()` public function to `crates/atlassy-adf/src/lib.rs` that sorts a `&mut [String]` of ADF paths by splitting on `/` and comparing segments numerically (when parseable as `usize`) or lexicographically (otherwise), with shorter paths before longer paths on shared prefix
- [ ] 1.3 Export `SCOPE_ANCHOR_TYPES` and `document_order_sort` from `atlassy-adf` crate public API

## 2. Discovery Filter Update

- [ ] 2.1 Update `discover_target_path()` in `crates/atlassy-adf/src/lib.rs` to add `!path_has_ancestor_type(path, node_path_index, SCOPE_ANCHOR_TYPES)` to the `Prose` candidate filter, after the existing `EDITABLE_PROSE_TYPES` check and before the table exclusion check
- [ ] 2.2 Replace `candidates.sort()` with `document_order_sort(&mut candidates)` in `discover_target_path()`

## 3. Scope Resolution Sort Update

- [ ] 3.1 Replace `matched_paths.sort()` with `document_order_sort(&mut matched_paths)` in `resolve_scope()` in `crates/atlassy-adf/src/lib.rs`

## 4. Unit Tests — atlassy-adf

- [ ] 4.1 Add unit test `document_order_sort_numeric_segments` verifying `/content/2/content/0` sorts before `/content/10/content/0`
- [ ] 4.2 Add unit test `document_order_sort_shared_prefix` verifying `/content/0` sorts before `/content/0/content/0`
- [ ] 4.3 Add unit test `scope_anchor_types_is_subset_of_editable_prose` verifying every entry in `SCOPE_ANCHOR_TYPES` exists in `EDITABLE_PROSE_TYPES`
- [ ] 4.4 Update test `discovers_first_prose_text_in_section` to assert `/content/1/content/0/text` (paragraph) instead of `/content/0/content/0/text` (heading)
- [ ] 4.5 Update test `discovers_nth_prose_text_with_index` to assert index 1 returns the second non-heading candidate
- [ ] 4.6 Update test `discovery_respects_scope_boundary` to assert `/content/3/content/0/text` (paragraph after out-of-scope heading) instead of `/content/2/content/0/text` (heading "Outside")
- [ ] 4.7 Add unit test `discovery_excludes_heading_text_nodes` with a section containing heading + paragraphs, verifying heading text is never in the candidate set
- [ ] 4.8 Add unit test `discovery_fails_for_heading_only_section` verifying `TargetDiscoveryFailed` with `found: 0` when scope contains only a heading with no paragraph content

## 5. Integration Tests — atlassy-pipeline

- [ ] 5.1 Update test `pipeline_auto_discovers_and_patches` to assert the discovered path is a paragraph text node, not a heading text node
- [ ] 5.2 Verify test `scoped_auto_discovery_finds_target_within_section` still passes with the heading exclusion (loose assert may already be compatible)

## 6. Build and Verify

- [ ] 6.1 Run `cargo build` and confirm no compilation errors
- [ ] 6.2 Run `cargo test` across all crates and confirm all tests pass (including updated tests)
