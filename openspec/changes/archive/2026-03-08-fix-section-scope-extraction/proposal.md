## Why

The scope resolver returns only the heading node, not the heading section. All 9 optimized KPI revalidation runs fail at `patch` with `ERR_SCHEMA_INVALID` because downstream `target_path` references point to paragraph nodes that do not exist in the heading-only scoped ADF. Additionally, the current design extracts a synthetic ADF subset for `scoped_adf`, which introduces an unsolved publish reintegration gap — publishing a scoped subset would overwrite the full page with only the section content. Both issues block the v1 readiness decision.

## What Changes

- **BREAKING**: `resolve_scope()` no longer extracts ADF subtrees into synthetic documents. `scoped_adf` always contains the full page ADF. Scope restriction is enforced purely via `allowed_scope_paths`.
- `resolve_scope()` gains section expansion: after finding a heading via `find_heading_paths()`, a new `expand_heading_to_section()` function walks sibling nodes forward, collecting paths until the next heading at equal or higher level (or end of content array).
- `context_reduction_ratio` is recomputed from section path bytes (`allowed_scope_paths` node sizes) vs full page bytes, replacing the previous `scoped_adf_bytes` vs `full_page_adf_bytes` formula.
- `scoped_adf_bytes` in `RunSummary` is repurposed to mean "section bytes" — the serialized size of only the nodes within `allowed_scope_paths`.
- The single-match bare-node branch and multi-match synthetic-doc branch in `resolve_scope()` are removed.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `scoped-adf-fetch-and-index`: Scope resolution now returns full page ADF with section-expanded `allowed_scope_paths` instead of extracting a scoped subtree. Section expansion uses heading-level-aware sibling walking.
- `scoped-payload-sizing`: `scoped_adf_bytes` now measures the serialized size of nodes within `allowed_scope_paths` (section bytes), not the size of the scoped ADF payload. `context_reduction_ratio` formula inputs change accordingly.

## Impact

- `crates/atlassy-adf`: `resolve_scope()` rewritten, new `expand_heading_to_section()` and `heading_level()` helpers added. Existing `find_heading_paths()` and `find_block_paths()` unchanged.
- `crates/atlassy-pipeline`: Telemetry computation block updated to compute `scoped_adf_bytes` from section paths. No structural changes to pipeline states or `FetchOutput`.
- `crates/atlassy-contracts`: No struct field additions or removals. `scoped_adf_bytes` semantic meaning changes (section bytes vs scoped ADF bytes).
- `crates/atlassy-cli`: `kpi_values()` function uses `scoped_adf_bytes` — semantics shift transparently. No code changes expected.
- Existing tests: Baseline behavior (empty `scope_selectors`) is preserved — `scoped_adf` was already the full page in that case. The `resolves_heading_scope` unit test must be updated for new `allowed_scope_paths` assertions.
