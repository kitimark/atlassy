## Why

The scoped pipeline path is broken end-to-end. After `fix-section-scope-extraction` changed `scoped_adf` to always contain the full page ADF, the `extract_prose` state was not updated to filter out-of-scope nodes. It iterates all `editable_prose` nodes from the full-page node manifest and fails with `ERR_SCOPE_MISS` when `canonicalize_mapped_path` encounters paths outside `allowed_scope_paths`. This blocks all scoped (heading-selector) operations, all KPI batch optimized runs, and all auto-discovery with heading scope. Additionally, heading selectors use substring matching (`text.contains()`) which could silently mis-scope content, and the `block:` selector has zero test coverage. All 26 pipeline integration tests override `scope_selectors` to `vec![]`, meaning the scoped path has never been tested end-to-end.

## What Changes

- Fix `extract_prose` to skip `editable_prose` nodes whose paths fall outside `allowed_scope_paths`, using a scope filter before `canonicalize_mapped_path`.
- Make `is_within_allowed_scope` public in `atlassy-adf` so the pipeline crate can use it for scope filtering.
- **BREAKING**: Change heading selector matching from substring (`text.contains(heading_text)`) to exact equality (`text == heading_text`) in `find_heading_paths()`. Selectors like `heading:View` will no longer match headings titled "Overview". All existing QA manifest selectors use full heading names and are unaffected.
- Add a new multi-section test fixture (`multi_section_adf.json`) with multiple heading sections to enable scoped path testing where in-scope and out-of-scope `editable_prose` nodes coexist.
- Add pipeline integration tests that exercise the scoped path with populated `scope_selectors`.
- Add unit tests for `block:` selector matching on `attrs.id` and `attrs.localId`.
- Add unit tests for duplicate heading multi-match behavior.
- Add unit tests enforcing the exact-match heading selector policy.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `prose-extraction-and-mapping`: The `extract_prose` state SHALL skip nodes outside `allowed_scope_paths` instead of failing on them. Adds a scope filtering requirement to the extraction loop.
- `scoped-adf-fetch-and-index`: Heading selectors SHALL use exact text match instead of substring match. Adds `block:` selector scenario coverage and duplicate heading multi-match behavior specification.

## Impact

- `crates/atlassy-adf`: `is_within_allowed_scope` visibility changes from `fn` to `pub fn`. `find_heading_paths` matching logic changes from `contains()` to `==`. No struct or trait changes.
- `crates/atlassy-pipeline`: `run_extract_prose_state()` gains a scope filter. Import list updated to include `is_within_allowed_scope`.
- `crates/atlassy-pipeline/tests`: New fixture file `multi_section_adf.json`. New integration test(s) for scoped pipeline path.
- `crates/atlassy-adf` tests: New unit tests for exact heading match, block selector, and duplicate heading behavior.
- No changes to `atlassy-contracts`, `atlassy-confluence`, or `atlassy-cli`.
