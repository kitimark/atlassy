# Scope Resolution Quality

## Status

Incubating

## Plain Problem Points

- The `heading:` scope selector uses substring matching (`text.contains(heading_text)`), not exact match. `heading:View` matches a heading titled "Overview".
- Duplicate headings with the same text produce multiple matched paths, triggering the multi-match synthetic wrapper path. This behavior is untested.
- The `block:` selector (matching on `attrs.id` or `attrs.localId`) has zero test coverage.
- Overall scope resolution has exactly 1 unit test with a non-empty selector (`heading:Overview`, single match). All 20+ pipeline integration tests override selectors to `[]`, exercising only the full-page fallback path.
- Edge cases are entirely untested: selector without `:`, unknown selector kind, heading not found fallback, nested headings, multi-selector combinations, colon in selector value.

## Proposed Direction

- Decide whether substring matching is intentional or a bug. If intentional, document it clearly and ensure heading naming guidance warns users. If unintentional, change `text.contains()` to exact equality check in `find_heading_paths()` (`crates/atlassy-adf/src/lib.rs`, line ~370).
- Add unit tests for: exact vs substring heading match, `block:` selector match on `attrs.id` and `attrs.localId`, not-found fallback, duplicate heading multi-match, malformed selector error, multi-selector merge.
- Add at least one pipeline integration test that uses non-empty `scope_selectors` and verifies `scope_resolution_failed: false` and `context_reduction_ratio > 0`.
- Update test fixtures (`batch_complete_manifest.json` and integration test helpers) to include runs with populated `scope_selectors`.

## Why Not Now

- KPI revalidation experiment will empirically test scope resolution against live Confluence ADF for the first time. Results may reveal additional issues that should inform the fix scope.
- Substring matching may be acceptable for v1 if heading names are chosen carefully (no substring overlaps on the same page).

## Risks

- Substring matching could cause silent incorrect scoping in enterprise content with common heading prefixes (e.g., "Introduction", "Introduction to Setup", "Introduction to Deployment").
- Lack of test coverage means regressions in scope resolution would go undetected.
- The `block:` selector has never been exercised in any context; it may have latent bugs.

## Signals To Revisit

- KPI revalidation spike reveals unexpected multi-match or wrong-heading scoping.
- Enterprise content testing surfaces substring collision issues.
- Any code change touches `resolve_scope` or `find_heading_paths`.

## Promotion Path

- Promote to a code fix change when KPI experiment results clarify whether substring matching is a practical problem.
- Test coverage expansion can be done independently of the matching behavior decision.

## Code References

- `resolve_scope`: `crates/atlassy-adf/src/lib.rs`, lines 48-95.
- `find_heading_paths` (substring match): `crates/atlassy-adf/src/lib.rs`, lines 360-383.
- `find_block_paths`: `crates/atlassy-adf/src/lib.rs`, lines 386-417.
- `full_page_resolution` (fallback): `crates/atlassy-adf/src/lib.rs`, lines 312-322.
- Single unit test: `crates/atlassy-adf/src/lib.rs`, lines 480-493.
