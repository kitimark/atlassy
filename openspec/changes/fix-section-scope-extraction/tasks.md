## 1. Section Expansion Helpers (atlassy-adf)

- [ ] 1.1 Add `heading_level(node: &Value) -> u8` helper in `crates/atlassy-adf/src/lib.rs` that extracts `attrs.level` from a heading node, defaulting to `6` if missing or invalid (D6)
- [ ] 1.2 Add `expand_heading_to_section(adf: &Value, heading_path: &str) -> Vec<String>` function that parses the heading index N from a `/content/N` path, reads the heading's level, walks the parent `content` array from index N+1 forward collecting sibling paths until the next heading at level <= heading level or end of array, and returns all collected paths including the heading itself (D2, D4)
- [ ] 1.3 In `expand_heading_to_section`, return an empty `Vec` if the heading path does not match the `/content/N` pattern (non-top-level heading), to support D5 fallback logic in `resolve_scope`

## 2. Rewrite resolve_scope (atlassy-adf)

- [ ] 2.1 Modify `resolve_scope()` at `crates/atlassy-adf/src/lib.rs:48-94`: after collecting heading paths from `find_heading_paths()`, call `expand_heading_to_section()` for each heading path and replace the heading path with the expanded section paths in `matched_paths`
- [ ] 2.2 If all heading paths returned empty expansions (non-top-level headings), fall back to full-page resolution with `fallback_reason: "nested_heading_scope_unsupported"` (D5)
- [ ] 2.3 Remove the single-match bare-node branch (lines 72-75) and the multi-match synthetic-doc branch (lines 76-84). Replace with: `scoped_adf = adf.clone()` — always return the full page ADF (D1)
- [ ] 2.4 Build `node_path_index` from the full page ADF and set `allowed_scope_paths` to the expanded section paths. Keep `scope_resolution_failed: false`, `full_page_fetch: false`

## 3. Unit Tests for Section Expansion (atlassy-adf)

- [ ] 3.1 Update existing `resolves_heading_scope` test: assert `allowed_scope_paths` is `["/content/0", "/content/1"]` (heading + trailing paragraph) and `scoped_adf` equals the full doc
- [ ] 3.2 Test: heading with trailing content until next same-level heading — h2 at 0, para at 1, para at 2, h2 at 3 → section paths `["/content/0", "/content/1", "/content/2"]`
- [ ] 3.3 Test: heading at end of content array — h2 as last node → section paths contain only the heading path
- [ ] 3.4 Test: adjacent same-level headings — h2 "A" at 0, h2 "B" at 1 → section for "A" is `["/content/0"]` only
- [ ] 3.5 Test: nested sub-headings included — h2 at 0, para at 1, h3 at 2, para at 3, h2 at 4 → section paths `["/content/0", "/content/1", "/content/2", "/content/3"]`
- [ ] 3.6 Test: h1 includes h2 and h3 subsections, stops at next h1
- [ ] 3.7 Test: multiple selectors produce union of section paths, deduplicated and sorted
- [ ] 3.8 Test: heading missing `attrs.level` defaults to level 6, stops at next heading of any level
- [ ] 3.9 Test: non-top-level heading (heading nested inside a panel node) falls back to full page with `fallback_reason` containing `"nested_heading_scope_unsupported"`

## 4. Metric Recomputation (atlassy-pipeline)

- [ ] 4.1 Add `compute_section_bytes(adf: &Value, section_paths: &[String]) -> u64` helper in `crates/atlassy-pipeline/src/lib.rs` that serializes each node at the given paths via `adf.pointer(path)` and sums compact-JSON byte lengths
- [ ] 4.2 Modify telemetry block at `crates/atlassy-pipeline/src/lib.rs:337-345`: replace `serde_json::to_vec(&fetch.payload.scoped_adf)` with `compute_section_bytes(&fetch.payload.scoped_adf, &fetch.payload.allowed_scope_paths)` for `scoped_adf_bytes`. Keep `context_reduction_ratio` formula unchanged (`1.0 - scoped_adf_bytes / full_page_adf_bytes`)
- [ ] 4.3 When `allowed_scope_paths` contains `"/"` (full-page fallback), `compute_section_bytes` SHALL return the full page byte size so `context_reduction_ratio` is `0.0`

## 5. Verification

- [ ] 5.1 `cargo fmt --all`
- [ ] 5.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 5.3 `cargo test --workspace` — all existing and new tests pass
- [ ] 5.4 Manual validation: run a single pipeline run with `--runtime-backend stub` and non-empty `scope_selectors` (e.g., `heading:Overview`), verify `context_reduction_ratio > 0`, `scope_resolution_failed: false`, and pipeline completes through `publish`
