## 1. Fix extract_prose scope filtering (atlassy-adf + atlassy-pipeline)

- [ ] 1.1 Make `is_within_allowed_scope` public in `crates/atlassy-adf/src/lib.rs:555` by changing `fn is_within_allowed_scope` to `pub fn is_within_allowed_scope` (D2)
- [ ] 1.2 Add `is_within_allowed_scope` to the import list in `crates/atlassy-pipeline/src/lib.rs:5-11`
- [ ] 1.3 Add `.filter(|node| is_within_allowed_scope(&node.path, &fetch.payload.allowed_scope_paths))` after the existing `.filter(|node| node.route == "editable_prose")` at `crates/atlassy-pipeline/src/lib.rs:612` (D1)

## 2. Fix heading selector exact match (atlassy-adf)

- [ ] 2.1 Change `text.contains(heading_text)` to `text == heading_text` in `find_heading_paths()` at `crates/atlassy-adf/src/lib.rs:482` (D3)

## 3. Unit tests for heading selector (atlassy-adf)

- [ ] 3.1 Add test `heading_selector_requires_exact_match`: ADF with heading "Overview", selector `heading:View` — assert `scope_resolution_failed: true` and `full_page_fetch: true` with fallback reason
- [ ] 3.2 Add test `heading_selector_exact_match_still_works`: ADF with heading "Overview", selector `heading:Overview` — assert resolution succeeds (verify existing tests still pass after the `contains` → `==` change)
- [ ] 3.3 Add test `duplicate_heading_text_matches_all_sections`: ADF with two h2 headings both titled "Notes" each followed by a paragraph, selector `heading:Notes` — assert `allowed_scope_paths` contains all four paths (both sections)

## 4. Unit tests for block selector (atlassy-adf)

- [ ] 4.1 Add test `resolves_block_scope_by_attrs_id`: ADF with a panel node having `attrs: {"id": "panel-1"}`, selector `block:panel-1` — assert path is in `allowed_scope_paths` and `scope_resolution_failed: false`
- [ ] 4.2 Add test `resolves_block_scope_by_attrs_local_id`: ADF with a node having `attrs: {"localId": "local-abc"}`, selector `block:local-abc` — assert path is in `allowed_scope_paths`
- [ ] 4.3 Add test `block_selector_falls_back_when_no_match`: ADF with no matching block ID, selector `block:nonexistent` — assert `scope_resolution_failed: true` and `full_page_fetch: true`

## 5. Multi-section test fixture (atlassy-pipeline)

- [ ] 5.1 Create `crates/atlassy-pipeline/tests/fixtures/multi_section_adf.json` with the structure from D4: h2 "Overview" + paragraph at `/content/0-1`, h2 "Details" + two paragraphs at `/content/2-4` — all with `attrs.level: 2` on headings

## 6. Scoped pipeline integration tests (atlassy-pipeline)

- [ ] 6.1 Add test `scoped_prose_update_only_touches_in_scope_paths`: use `sample_request()` with default `scope_selectors` (no override to `vec![]`), `multi_section_adf.json` fixture, `SimpleScopedProseUpdate` targeting `/content/1/content/0/text` — assert run succeeds, `applied_paths` contains only the in-scope path, and publish happens once
- [ ] 6.2 Add test `scoped_auto_discovery_finds_target_within_section`: use `sample_request()` with default `scope_selectors`, `multi_section_adf.json` fixture, `SimpleScopedProseUpdate` with `target_path: None` — assert `discovered_target_path` is within `/content/0` or `/content/1` (the Overview section), not in `/content/2-4`

## 7. Verification

- [ ] 7.1 `cargo fmt --all`
- [ ] 7.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 7.3 `cargo test --workspace` — all existing and new tests pass
