## Context

`resolve_scope()` in `crates/atlassy-adf/src/lib.rs:48-94` has two related defects that block the v1 readiness decision:

1. **Section extraction missing.** `find_heading_paths()` returns the path of the heading node itself (e.g., `/content/0`). `resolve_scope()` extracts just that node. Downstream states reference paths like `/content/1/content/0/text` (a paragraph after the heading) which don't exist in the heading-only scoped ADF, causing `ERR_SCHEMA_INVALID` at the patch stage. All 9 optimized KPI runs failed this way (see `qa/investigations/2026-03-08-kpi-revalidation.md`).

2. **Publish reintegration gap.** The single-match branch (`:72-75`) returns a bare node as `scoped_adf`. The multi-match branch (`:76-84`) wraps extracted nodes in a synthetic `{ type: "doc", content: [...] }`. In both cases, `run_publish_state()` at `:1154` publishes `candidate_page_adf` (derived from `scoped_adf`) as the entire page body. If `scoped_adf` is a subset, publishing it destroys all content outside the scope. This was never triggered because optimized runs fail before reaching publish, and baseline runs use empty selectors (full page).

Additionally, the multi-match branch re-indexes nodes into the synthetic doc (e.g., original paths `/content/3`, `/content/4` become `/content/0`, `/content/1`), but `allowed_scope_paths` retains the original indices, creating a path mismatch that would break all downstream scope checks.

The pipeline stores only `full_page_adf_bytes` (a metric) in `FetchOutput`, not the actual full page ADF. There is no mechanism to merge a patched scope subset back into the full page.

**Goals:**

- Fix section extraction so `allowed_scope_paths` includes heading + section body nodes.
- Eliminate the path remapping and publish reintegration problems.
- Preserve the `context_reduction_ratio` KPI with a meaningful computation.
- Keep all existing baseline tests passing without modification.
- Minimize change surface for a v1 PoC.

**Non-Goals:**

- Reducing the byte size of `scoped_adf` in memory. Token savings are enacted at the markdown extraction layer (`md_assist_edit`), not at the ADF struct level.
- Supporting section extraction for headings nested inside containers (panels, layouts, expand blocks). This is deferred to post-v1.
- Changing `find_heading_paths()` or `find_block_paths()` function signatures.
- Adding new fields to `FetchOutput` or `RunSummary`.

## Decisions

### D1: Full page ADF as scoped_adf (Approach C)

**Decision:** `scoped_adf` always contains the full page ADF. Scope restriction is enforced purely through `allowed_scope_paths`. The single-match bare-node branch and multi-match synthetic-doc branch are removed.

**Rationale:** This eliminates three problems simultaneously: (a) path remapping — all paths reference into the real document at all times, (b) publish reintegration — `candidate_page_adf` is the full page, so publishing is safe, (c) the `canonicalize_mapped_path()` re-rooting logic becomes unnecessary for scoped runs since paths are already canonical. The real token savings happen at `md_assist_edit` where only section markdown is extracted for the AI, not at the ADF struct level. ADF processing is microsecond-cost Rust, not LLM token cost.

**Alternatives considered:**
- *Approach A (sibling walk with synthetic extraction):* Smallest code change in isolation, but the synthetic doc re-indexes nodes, breaking path coherence when the heading isn't at `/content/0`. Requires path remapping to fix, which inflates the change to Approach B's size. Does not solve publish reintegration.
- *Approach B (always-doc wrapper with remap table):* Fixes path coherence via an explicit remap table, but introduces a new first-class concept the pipeline doesn't have. Still requires storing full page ADF in `FetchOutput` for publish reintegration. Largest change surface, most opportunity for bugs in a v1 PoC.

### D2: Section boundaries by heading level

**Decision:** `expand_heading_to_section()` walks the parent content array forward from the heading's index, collecting sibling paths until it encounters a heading at equal or higher level (level number <= heading's level), or reaches the end of the array. Sub-headings (higher level numbers) are included as part of the section.

**Rationale:** Matches Markdown section semantics where `## Introduction` owns everything until the next `##` or `#`, including any `###` subsections. This is what users expect when they say "edit the Introduction section."

**Alternatives considered:**
- *Stop at any heading regardless of level:* Would split natural document sections. A section titled "Architecture" with subsections "Frontend" and "Backend" would lose the subsections.
- *Explicit end selector (e.g., `heading:Introduction..heading:Details`):* More flexible but adds API complexity without clear v1 need. Can be added later.

### D3: Recompute context_reduction_ratio from section paths

**Decision:** Compute `scoped_adf_bytes` by serializing each node at `allowed_scope_paths` and summing byte lengths. `context_reduction_ratio` formula stays `1.0 - (scoped_adf_bytes / full_page_adf_bytes)` but now uses section bytes as the numerator. A new helper `compute_section_bytes(adf, section_paths) -> u64` in the pipeline handles this.

**Rationale:** Preserves the KPI's intent — "what fraction of the page are we actually touching?" — without requiring a physically smaller `scoped_adf`. The 70% gate threshold remains meaningful: a heading section that is 30% of the page will report 70% reduction.

**Alternatives considered:**
- *Report 0% and rely on `scoped_section_tokens`:* Honest but breaks the KPI gate at 70%, which is load-bearing in the readiness checklist. Would require changing gate thresholds and spec language simultaneously.

### D4: Section expansion as a separate function

**Decision:** `find_heading_paths()` continues to return only heading paths. A new `expand_heading_to_section(adf, heading_path) -> Vec<String>` function handles expansion. `resolve_scope()` calls expand after find.

**Rationale:** Separation of concerns. `find_heading_paths` is a pure locator. Section expansion is a distinct concern with its own edge cases and test surface. Both are independently testable.

### D5: Non-top-level headings fall back to full page

**Decision:** `expand_heading_to_section()` only operates on headings that are direct children of the document's top-level `content` array (paths matching `/content/N`). If a heading is nested inside a panel, layout, or other container, section expansion is not attempted and `resolve_scope()` falls back to full-page scope with `fallback_reason: "nested_heading_scope_unsupported"`.

**Rationale:** Walking siblings of a nested heading requires understanding container boundaries — a panel's content array is separate from the document's content array. Getting this wrong could include nodes from sibling containers. Top-level heading sections cover the vast majority of real Confluence pages. Nested heading support can be added post-v1 without breaking changes.

### D6: heading_level defaults to 6

**Decision:** The `heading_level(node)` helper returns `attrs.level` as `u8`, defaulting to `6` if the attribute is missing or not a valid number.

**Rationale:** Level 6 is the lowest (most deeply nested) heading level. Defaulting to 6 means a heading with missing level stops at the next heading of any level — the most conservative behavior. This prevents accidentally including too much content.

## Risks / Trade-offs

- **[Risk] `scoped_adf` is now always the full page, so `scoped_adf_bytes` equals `full_page_adf_bytes` in the raw payload.** Mitigation: `scoped_adf_bytes` in `RunSummary` is recomputed from section paths, not from the `scoped_adf` payload size. The telemetry accurately reflects scope coverage.

- **[Risk] Heading text matching uses `contains()` (substring) at `find_heading_paths():365`.** A selector `heading:Intro` would match both "Introduction" and "Introducing New Features". Mitigation: This is a pre-existing issue tracked in `ideas/2026-03-scope-resolution-quality.md` and is out of scope for this change. The KPI revalidation investigation confirmed no accidental cross-matches occurred in the 9 optimized runs.

- **[Risk] Section expansion assumes flat top-level content array.** ADF documents with all content inside layout or section containers would have no top-level headings, causing all selectors to fall back to full page. Mitigation: Fallback to full page is safe (identical to baseline behavior). This is an inherent limitation of v1 scope, documented in D5.

- **[Risk] Existing `resolves_heading_scope` test asserts `allowed_scope_paths == ["/content/0"]`.** This assertion must change to include section body paths. Mitigation: The test fixture has a heading at `/content/0` and a paragraph at `/content/1`, so the updated assertion is `["/content/0", "/content/1"]` — the full section.
