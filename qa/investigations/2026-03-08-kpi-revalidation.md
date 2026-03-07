# 2026-03-08 KPI Revalidation Investigation

## Provenance

- `git_commit_sha`: `0e690674fadad849aa0fe201704a31e49350f797`
- `git_dirty`: `true`
- `pipeline_version`: `v1`
- `runtime_mode`: `live`
- Evidence bundle: `qa/evidence/2026-03-08-kpi-revalidation/`
- Manifest: `qa/manifests/kpi-revalidation-batch.json`

## Objective

Run the first paired baseline/optimized KPI experiment across all three edit patterns (A, B, C) to produce a defensible `go/iterate/stop` recommendation. The readiness status has been `iterate` since the KPI framework revision (D-014) because all prior runs used empty `scope_selectors`, meaning the scoped-vs-full-page comparison was never exercised.

## Experiment Design

- 3 sandbox pages with structural variety: P1 (prose-rich, Pattern A), P2 (prose+table, Pattern B), P3 (prose near locked blocks, Pattern C).
- 18 runs total: 3 baseline/optimized pairs per pattern (9 pairs).
- Baseline runs use `scope_selectors: []` (full-page fetch). Optimized runs use explicit heading selectors.
- Runs paired by `(page_id, pattern, edit_intent_hash)`.

## Results

### Overall

| Metric | Value |
|--------|-------|
| Total runs | 18 |
| Baseline succeeded | 9/9 (100%) |
| Optimized succeeded | 0/9 (0%) |
| Pairs formed | 0/9 |
| Recommendation | **iterate** |
| Blocking gate | `paired_matrix_complete` |

### Baseline Performance (9/9 success)

| Run ID | Page | Pattern | Latency | Publish |
|--------|------|---------|---------|---------|
| kpi-a-base-01 | P1 | A | 1709ms | published (v4) |
| kpi-a-base-02 | P1 | A | 1863ms | published (v5) |
| kpi-a-base-03 | P1 | A | 1716ms | published (v6) |
| kpi-b-base-01 | P2 | B | 1666ms | published (v4) |
| kpi-b-base-02 | P2 | B | 1705ms | published (v5) |
| kpi-b-base-03 | P2 | B | 1607ms | published (v6) |
| kpi-c-base-01 | P3 | C | 1876ms | published (v4) |
| kpi-c-base-02 | P3 | C | 1840ms | published (v5) |
| kpi-c-base-03 | P3 | C | 1733ms | published (v6) |

Baseline summary:
- Median latency: ~1716ms. All below 3000ms threshold.
- Zero retries across all runs.
- Zero locked-node mutations.
- Edit success rate: 100%.

### Optimized Failures (0/9 success)

| Run ID | Page | Pattern | Failure | Error | Reduction |
|--------|------|---------|---------|-------|-----------|
| kpi-a-opt-01 | P1 | A | patch | ERR_SCHEMA_INVALID | 96.1% |
| kpi-a-opt-02 | P1 | A | patch | ERR_SCHEMA_INVALID | 96.3% |
| kpi-a-opt-03 | P1 | A | patch | ERR_SCHEMA_INVALID | 96.0% |
| kpi-b-opt-01 | P2 | B | patch | ERR_SCHEMA_INVALID | 96.3% |
| kpi-b-opt-02 | P2 | B | adf_table_edit | ERR_ROUTE_VIOLATION | 96.5% |
| kpi-b-opt-03 | P2 | B | patch | ERR_SCHEMA_INVALID | 96.4% |
| kpi-c-opt-01 | P3 | C | patch | ERR_SCHEMA_INVALID | 96.1% |
| kpi-c-opt-02 | P3 | C | patch | ERR_SCHEMA_INVALID | 95.9% |
| kpi-c-opt-03 | P3 | C | patch | ERR_SCHEMA_INVALID | 95.4% |

## Root Cause

**The scope resolver returns the heading node itself, not the heading section.**

`resolve_scope()` at `crates/atlassy-adf/src/lib.rs:48-95`:

1. `find_heading_paths()` locates the heading matching the selector text (e.g., "Introduction" at path `/content/0` in the full-page ADF).
2. For a single match, `pointer_get(adf, &matched_paths[0])` extracts **just the heading node** — a ~88-byte object containing `{type: "heading", content: [{text: "Introduction", type: "text"}]}`.
3. The `scoped_adf` returned to downstream pipeline states contains only this heading node.
4. The manifest's `target_path` (e.g., `/content/1/content/0/text`) references the paragraph **after** the heading in the full-page ADF. This path does not exist in the heading-only scoped ADF.
5. The patch state tries to resolve `target_path` against the scoped ADF, fails, and emits `ERR_SCHEMA_INVALID`.

The one `ERR_ROUTE_VIOLATION` on `kpi-b-opt-02` is the same root cause: the table cell path doesn't exist in a heading-only scoped ADF, and the `adf_table_edit` state rejects it before reaching patch.

### Why the Scoped Fetch Spike Did Not Catch This

The Phase 2 spike runs used `--force-verify-fail` and `--mode no-op`. These flags cause the pipeline to:
- Succeed at `fetch` (scope resolution works — headings are found).
- Emit correct scope metrics (`scope_resolution_failed: false`, `context_reduction_ratio > 0`).
- Skip `patch`, `verify`, and `publish` states entirely.

The spike validated scope **resolution** but not scope **utilization**. The defect only manifests when the pipeline attempts to apply edits against the scoped ADF.

## Defect: Section Extraction Required

**What `resolve_scope` should do**: Given `heading:Introduction`, return the heading plus all subsequent sibling content nodes until the next heading (or end of parent array). This is "section semantics" — a heading-delimited section of the document.

**Example**: For a full-page ADF with structure:
```
/content/0: heading "Introduction"
/content/1: paragraph (first para under Introduction)
/content/2: paragraph (second para under Introduction)
/content/3: heading "Details"
/content/4: bulletList ...
```

Current behavior: returns just `/content/0` (the heading node, 88 bytes).
Required behavior: returns `/content/0`, `/content/1`, `/content/2` (heading + section content, ~400+ bytes).

**Location**: `crates/atlassy-adf/src/lib.rs`, `find_heading_paths()` at lines 360-383 and `resolve_scope()` at lines 48-95.

**Fix approach**: After `find_heading_paths()` returns a heading path like `/content/N`, walk the parent array from index `N+1` forward, collecting sibling paths until the next heading node or end of array. Include all collected paths in `matched_paths` before building the scoped ADF.

## KPI Gate Assessment

| KPI | Threshold | Result | Notes |
|-----|-----------|--------|-------|
| `context_reduction_ratio` | Optimized median >= 70% | **Inconclusive** | Reduction metrics look promising (95-96%) but runs failed before publish |
| `edit_success_rate` | > 95% | **Inconclusive** | Baseline 100%, optimized 0% — defect, not a real signal |
| `structural_preservation` | 100% | **Pass (baseline only)** | Zero locked-node mutations across all 18 runs |
| `conflict_rate` | < 10% | **Pass (baseline only)** | Zero retries across all 18 runs |
| `publish_latency` | Optimized median < 3000ms | **Pass (baseline only)** | Baseline median ~1716ms |

## Positive Signals

Despite the blocking defect, the experiment produced useful evidence:

1. **Full-page baseline pipeline is solid.** 9/9 baseline runs published successfully with zero retries, zero safety violations, and latencies well within targets.
2. **Scope resolution finds headings correctly.** All 9 optimized runs resolved their heading selectors (`scope_resolution_failed: false`). The substring matching concern from `ideas/2026-03-scope-resolution-quality.md` was not triggered — no accidental cross-matches.
3. **Context reduction is real.** Optimized runs achieved 95.4-96.5% reduction ratios. Even though the runs failed at patch, the fetch-stage reduction is valid and well above the 70% target.
4. **Safety gates held.** Zero locked-node violations, zero out-of-scope violations, zero table-shape violations. The safety layer correctly blocked invalid paths.
5. **KPI infrastructure works end-to-end.** Telemetry emission, batch reporting, gate checks, pairing logic, and recommendation generation all functioned correctly. The `kpi: null` output is correct behavior when pairs are incomplete.

## Recommendation

**iterate** — fix the scope resolver section extraction defect, then re-run the 18-run batch.

### Handover for Fix

1. Implement section extraction in `resolve_scope()` / `find_heading_paths()`.
2. Add unit tests for section extraction (heading + content until next heading).
3. Re-seed experiment pages (baseline publishes modified page content to version 6).
4. Re-discover target paths via scoped fetch spike with updated resolver.
5. Update manifest with new target paths.
6. Re-run the 18-run batch with `--runtime-backend live`.

### Open Items

- `ideas/2026-03-scope-resolution-quality.md`: update with section extraction defect (this investigation supersedes the substring-matching concern).
- `ideas/2026-03-route-classification-drift.md`: `rule` node classification remains an open item, not related to this defect.
- Pages P1/P2/P3 are now at version 6 from baseline publishes. Content has been modified. Re-seeding or re-discovery of target paths is needed for the next batch.
