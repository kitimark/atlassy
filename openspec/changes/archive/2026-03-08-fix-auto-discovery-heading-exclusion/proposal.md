## Why

Auto-discovery selects heading text nodes as edit targets, which destroys heading-based scope selectors when baseline runs overwrite the heading text. This caused 3/9 optimized KPI batch runs to fall back to full-page fetch (0% context reduction) in the v2 revalidation. The root cause is that `EDITABLE_PROSE_TYPES` serves dual duty as both a route classifier ("is this editable?") and a discovery filter ("should this be auto-selected?"), conflating editability with discoverability. A secondary issue is that discovery uses string sort for candidate ordering, producing non-document-order results when content indices exceed single digits.

## What Changes

- Add `SCOPE_ANCHOR_TYPES` constant (`["heading"]`) to `atlassy-adf`. Heading nodes are editable prose but their text content serves as structural identifiers for heading-based scope selectors. Auto-discovery must exclude them.
- **BREAKING** (discovery behavior): `discover_target_path()` with `TargetRoute::Prose` and `target_index: 0` will return the first paragraph/list/blockquote text node instead of the first heading text node. Manifests relying on index 0 to select heading text must use explicit `target_path` instead.
- Add `document_order_sort()` function for natural numeric sorting of ADF content paths. Replace string sort with document-order sort in `discover_target_path()`, `resolve_scope()`, and `extract_prose` path collections for consistent ordering throughout the pipeline.
- Update existing unit and integration tests that assert heading-text-first discovery behavior to assert paragraph-text-first behavior.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `target-path-auto-discovery`: Discovery filter must exclude text nodes under `SCOPE_ANCHOR_TYPES` ancestors. Candidate sort must use document-order (natural numeric) instead of lexicographic string sort.
- `scoped-adf-fetch-and-index`: Add `SCOPE_ANCHOR_TYPES` constant definition alongside `EDITABLE_PROSE_TYPES`. Document-order sort for `allowed_scope_paths` and `node_path_index`-derived path collections.

## Impact

- `crates/atlassy-adf/src/lib.rs`: New constant, new sort function, updated `discover_target_path()` filter and sort, updated `resolve_scope()` sort.
- `crates/atlassy-pipeline/src/lib.rs`: Updated sort in `run_extract_prose_state()`, updated imports.
- `crates/atlassy-adf/src/lib.rs` (tests): ~5 unit tests updated or added.
- `crates/atlassy-pipeline/tests/pipeline_integration.rs`: ~2 integration tests updated.
- `openspec/specs/target-path-auto-discovery/spec.md`: Delta spec with new requirements and updated scenarios.
- `openspec/specs/scoped-adf-fetch-and-index/spec.md`: Delta spec for `SCOPE_ANCHOR_TYPES` and document-order sort.
- No API changes, no CLI changes, no manifest format changes. Explicit `target_path` behavior is unchanged.
