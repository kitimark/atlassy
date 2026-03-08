## Context

The `fix-section-scope-extraction` change (commit `86cf652`) refactored `resolve_scope()` so that `scoped_adf` always contains the full page ADF and scope restriction is enforced purely via `allowed_scope_paths`. This was the correct architectural decision (eliminates path remapping, publish reintegration, and synthetic doc problems), but it introduced a contract shift: the `classify` state now produces a `node_manifest` containing ALL page nodes, including those outside `allowed_scope_paths`.

The `extract_prose` state at `crates/atlassy-pipeline/src/lib.rs:608-616` iterates all `editable_prose` nodes and calls `canonicalize_mapped_path()` on each. That function returns `AdfError::OutOfScope` for paths not in `allowed_scope_paths`, which `to_hard_error()` converts to a fatal `ERR_SCOPE_MISS`. The state should skip out-of-scope nodes, not fail on them.

Separately, `find_heading_paths()` at `crates/atlassy-adf/src/lib.rs:482` uses `text.contains(heading_text)` for matching. Decision D-015 in the roadmap specifies exact match. The `block:` selector (`find_block_paths`) has never been exercised by any test. All 26 pipeline integration tests override `scope_selectors` to empty, so the scoped pipeline path has zero end-to-end test coverage.

## Goals / Non-Goals

**Goals:**

- Unblock scoped pipeline operations by fixing the `extract_prose` scope filter.
- Enforce exact heading selector matching per D-015.
- Establish end-to-end test coverage for the scoped pipeline path.
- Cover `block:` selector and duplicate heading edge cases with unit tests.

**Non-Goals:**

- Supporting nested heading scope (headings inside panels/layouts). Deferred post-v1 per D5 in `fix-section-scope-extraction`.
- Changing `canonicalize_mapped_path()` behavior. The function correctly rejects out-of-scope paths; the caller should not pass them.
- Adding whitespace trimming to heading matching. Exact match means exact.
- Modifying `classify` to only emit in-scope nodes. The full-page manifest is correct; filtering is the consumer's responsibility.

## Decisions

### D1: Filter at the consumer, not the producer

**Decision:** Add a `.filter()` call in `run_extract_prose_state()` to skip out-of-scope nodes, rather than modifying `classify` to exclude them.

**Rationale:** The classify state produces a complete node manifest of the page — this is correct and useful for diagnostics, telemetry, and future states that may need full-page visibility. Scope filtering is the responsibility of each downstream consumer that operates on scoped data. This follows the same pattern as `build_patch_ops()` which already checks `is_within_allowed_scope` before building patch operations.

**Alternatives considered:**
- *Filter in classify:* Would reduce downstream filtering needs but loses full-page visibility. Classify would need `allowed_scope_paths` as input, adding a dependency on fetch output that doesn't currently exist. Also breaks the clean separation where classify is purely structural and scope is a fetch concern.

### D2: Make `is_within_allowed_scope` public

**Decision:** Change `fn is_within_allowed_scope` to `pub fn is_within_allowed_scope` in `crates/atlassy-adf/src/lib.rs:555`.

**Rationale:** The function is already used internally by three public functions (`canonicalize_mapped_path`, `ensure_paths_in_scope`, `build_patch_ops`). Making it public lets the pipeline crate use it directly for the scope filter. The alternative — reimplementing the logic inline or using `is_path_within_or_descendant` with a manual `/` check — would duplicate logic.

### D3: Exact heading match, no trim

**Decision:** Change `text.contains(heading_text)` to `text == heading_text` in `find_heading_paths()` at line 482.

**Rationale:** D-015 in `roadmap/06-decisions-and-defaults.md` already decided on exact match. Substring matching risks silent mis-scoping in enterprise content with common heading prefixes ("Introduction" vs "Introduction to Setup"). All existing QA selectors use full heading names. No trim — whitespace in heading text is part of the heading identity. Users who need fuzzy matching can request it as a future enhancement.

**Alternatives considered:**
- *Exact match with trim:* More forgiving for messy Confluence content, but introduces ambiguity about which whitespace is significant. Confluence heading text nodes generally don't have leading/trailing whitespace.
- *Keep substring, add exact-match option:* Backward compatible but perpetuates the latent risk. No existing users depend on substring behavior.

### D4: New multi-section fixture for scoped integration tests

**Decision:** Create a new fixture `multi_section_adf.json` with two heading sections rather than extending existing fixtures.

**Rationale:** Existing fixtures (`prose_only_adf.json`, `page_adf.json`) have exactly one heading section. All 26 existing integration tests assert on specific paths within these fixtures. Extending them would break existing path assertions and conflate unrelated test concerns. A new fixture isolates the scoped-path tests cleanly.

The fixture structure:

```
/content/0: heading "Overview"      ← heading:Overview scope target
/content/1: paragraph "Overview body"
/content/2: heading "Details"       ← boundary (same level, ends Overview section)
/content/3: paragraph "Details body"
/content/4: paragraph "More details"
```

Scoping to `heading:Overview` produces `allowed_scope_paths: ["/content/0", "/content/1"]`. Nodes at `/content/2` through `/content/4` are out of scope but still classified as `editable_prose` in the full-page manifest — exactly the conditions that trigger the bug.

### D5: Scoped integration test approach

**Decision:** Add integration tests that use `sample_request()` without overriding `scope_selectors` (which defaults to `["heading:Overview"]`), paired with the multi-section fixture.

**Rationale:** The existing `sample_request()` helper already sets `scope_selectors: vec!["heading:Overview".to_string()]` at line 36 of `pipeline_integration.rs`. Every existing test overrides this to `vec![]`. By not overriding, we exercise the scoped path with minimal test code. The multi-section fixture ensures there are out-of-scope nodes to filter.

Tests to add:
- Scoped prose update succeeds and only touches in-scope paths.
- Scoped auto-discovery finds targets within the scoped section only.

## Risks / Trade-offs

- **[Risk] Heading exact-match is a breaking change for substring-dependent selectors.** Mitigation: All existing QA manifest selectors confirmed to use full heading names. No external users exist (pre-release v1 PoC). D-015 already approved this change.

- **[Risk] `extract_prose` silently skips out-of-scope nodes instead of reporting them.** Mitigation: This is intentional — scope filtering is a normal operation, not an error. The full node manifest is preserved in the classify output artifacts for diagnostics. The `extract_prose` output naturally reflects only in-scope nodes, which is visible in the `markdown_blocks` count.

- **[Risk] `block:` selector tests may reveal latent bugs in `find_block_paths`.** Mitigation: The function is 30 lines with straightforward logic (check `attrs.id` and `attrs.localId`). If bugs are found during testing, they should be fixed as part of this change rather than deferred.
