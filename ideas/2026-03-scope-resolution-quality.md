# Scope Resolution Quality

## Status

Incubating — elevated to blocking defect after KPI revalidation (2026-03-08).

## Blocking Defect: Section Extraction

**Discovered**: 2026-03-08 KPI revalidation batch. All 9 optimized runs failed.
**Evidence**: `qa/evidence/2026-03-08-kpi-revalidation/`, `qa/investigations/2026-03-08-kpi-revalidation.md`.

`resolve_scope()` returns the heading node itself, not the heading section (heading + subsequent sibling content until the next heading). When `heading:Introduction` matches, the scoped ADF is just `{type: "heading", content: [{text: "Introduction"}]}` (~88 bytes). Pipeline `target_path` values referencing paragraphs after the heading do not exist in this scoped ADF, causing `ERR_SCHEMA_INVALID` at the `patch` state.

**Required fix**: After `find_heading_paths()` returns a heading path like `/content/N`, walk the parent array from index `N+1` forward, collecting sibling paths until the next heading node or end of array. Include all collected paths in `matched_paths`.

**Location**: `crates/atlassy-adf/src/lib.rs`, `find_heading_paths()` lines 360-383, `resolve_scope()` lines 48-95.

## Other Problem Points

- The `heading:` scope selector uses substring matching (`text.contains(heading_text)`), not exact match. `heading:View` matches a heading titled "Overview". KPI revalidation did not trigger any substring collisions (heading names were chosen carefully), but this remains a latent risk.
- Duplicate headings with the same text produce multiple matched paths, triggering the multi-match synthetic wrapper path. This behavior is untested.
- The `block:` selector (matching on `attrs.id` or `attrs.localId`) has zero test coverage.
- Overall scope resolution has exactly 1 unit test with a non-empty selector (`heading:Overview`, single match). All 20+ pipeline integration tests override selectors to `[]`, exercising only the full-page fallback path.
- Edge cases are entirely untested: selector without `:`, unknown selector kind, heading not found fallback, nested headings, multi-selector combinations, colon in selector value.

## Proposed Direction

- **Priority 1**: Implement section extraction in `resolve_scope()` — this is the blocking defect. Heading selectors should return the heading plus all subsequent sibling content until the next heading or end of parent array.
- **Priority 2**: Add unit tests for section extraction: single heading with trailing content, heading at end of array, adjacent headings (empty section), nested content under heading.
- **Priority 3**: Decide whether substring matching is intentional or a bug. If intentional, document it clearly and ensure heading naming guidance warns users. If unintentional, change `text.contains()` to exact equality check in `find_heading_paths()` (`crates/atlassy-adf/src/lib.rs`, line ~370).
- Add unit tests for: exact vs substring heading match, `block:` selector match on `attrs.id` and `attrs.localId`, not-found fallback, duplicate heading multi-match, malformed selector error, multi-selector merge.
- Add at least one pipeline integration test that uses non-empty `scope_selectors` and verifies `scope_resolution_failed: false` and `context_reduction_ratio > 0`.
- Update test fixtures (`batch_complete_manifest.json` and integration test helpers) to include runs with populated `scope_selectors`.

## Why Not Now

- Section extraction is blocking and should be promoted to a code change immediately.
- Substring matching was not triggered during the KPI revalidation (heading names were chosen to avoid overlaps). It may be acceptable for v1 if heading names are chosen carefully.

## Risks

- Substring matching could cause silent incorrect scoping in enterprise content with common heading prefixes (e.g., "Introduction", "Introduction to Setup", "Introduction to Deployment").
- Lack of test coverage means regressions in scope resolution would go undetected.
- The `block:` selector has never been exercised in any context; it may have latent bugs.

## Signals To Revisit

- Section extraction fix is validated and KPI revalidation re-run passes.
- Enterprise content testing surfaces substring collision issues.
- Any code change touches `resolve_scope` or `find_heading_paths`.

## Promotion Path

- Section extraction defect: **promote immediately** to a code change. This blocks KPI revalidation.
- Substring matching and test coverage: promote after section extraction fix is landed and KPI revalidation re-run is complete.

## Code References

- `resolve_scope`: `crates/atlassy-adf/src/lib.rs`, lines 48-95.
- `find_heading_paths` (substring match): `crates/atlassy-adf/src/lib.rs`, lines 360-383.
- `find_block_paths`: `crates/atlassy-adf/src/lib.rs`, lines 386-417.
- `full_page_resolution` (fallback): `crates/atlassy-adf/src/lib.rs`, lines 312-322.
- Single unit test: `crates/atlassy-adf/src/lib.rs`, lines 480-493.
