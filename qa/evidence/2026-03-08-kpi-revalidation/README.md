# 2026-03-08 KPI Revalidation Evidence Bundle

This bundle captures the first live KPI revalidation batch (18 runs across patterns A, B, C) against 3 structurally varied sandbox pages. The batch exposed a blocking defect in the scope resolver that prevents all optimized runs from completing.

## Provenance

- `git_commit_sha`: `0e690674fadad849aa0fe201704a31e49350f797`
- `git_dirty`: `true` (uncommitted manifest and inventory changes)
- `runtime_mode`: `live`
- `pipeline_version`: `v1`
- Sandbox pages: P1=`65934` (prose-rich), P2=`98323` (prose+table), P3=`131227` (locked-adjacent)
- Parent page: `131207`

## Batch Summary

- Total runs: 18 (9 baseline, 9 optimized)
- Baseline succeeded: **9/9** (100%)
- Optimized succeeded: **0/9** (0%)
- Recommendation: **iterate**
- Blocking gate: `paired_matrix_complete` (failed — no optimized runs completed)

## Included Runs

### Baseline Runs (all succeeded)

| Run ID | Page | Pattern | Latency | Version | Publish |
|--------|------|---------|---------|---------|---------|
| `kpi-a-base-01` | P1 (65934) | A | 1709ms | 4 | published |
| `kpi-a-base-02` | P1 (65934) | A | 1863ms | 5 | published |
| `kpi-a-base-03` | P1 (65934) | A | 1716ms | 6 | published |
| `kpi-b-base-01` | P2 (98323) | B | 1666ms | 4 | published |
| `kpi-b-base-02` | P2 (98323) | B | 1705ms | 5 | published |
| `kpi-b-base-03` | P2 (98323) | B | 1607ms | 6 | published |
| `kpi-c-base-01` | P3 (131227) | C | 1876ms | 4 | published |
| `kpi-c-base-02` | P3 (131227) | C | 1840ms | 5 | published |
| `kpi-c-base-03` | P3 (131227) | C | 1733ms | 6 | published |

Baseline observations:
- `scope_selectors: []` → `full_page_fetch: true`, `context_reduction_ratio: 0.0` (expected).
- Median latency: ~1716ms. All within 3000ms target.
- Pages progressed from version 3→4→5→6 across 3 sequential publishes each.

### Optimized Runs (all failed)

| Run ID | Page | Pattern | Failure State | Error Code | Context Reduction |
|--------|------|---------|---------------|------------|-------------------|
| `kpi-a-opt-01` | P1 (65934) | A | `patch` | `ERR_SCHEMA_INVALID` | 96.1% |
| `kpi-a-opt-02` | P1 (65934) | A | `patch` | `ERR_SCHEMA_INVALID` | 96.3% |
| `kpi-a-opt-03` | P1 (65934) | A | `patch` | `ERR_SCHEMA_INVALID` | 96.0% |
| `kpi-b-opt-01` | P2 (98323) | B | `patch` | `ERR_SCHEMA_INVALID` | 96.3% |
| `kpi-b-opt-02` | P2 (98323) | B | `adf_table_edit` | `ERR_ROUTE_VIOLATION` | 96.5% |
| `kpi-b-opt-03` | P2 (98323) | B | `patch` | `ERR_SCHEMA_INVALID` | 96.4% |
| `kpi-c-opt-01` | P3 (131227) | C | `patch` | `ERR_SCHEMA_INVALID` | 96.1% |
| `kpi-c-opt-02` | P3 (131227) | C | `patch` | `ERR_SCHEMA_INVALID` | 95.9% |
| `kpi-c-opt-03` | P3 (131227) | C | `patch` | `ERR_SCHEMA_INVALID` | 95.4% |

Optimized observations:
- Scope resolution succeeded on all 9 runs (`scope_resolution_failed: false`).
- Context reduction ratios are 95.4-96.5% — well above the 70% target.
- All failed because `target_path` references paths valid in the full-page ADF but absent in the scoped ADF.
- `kpi-b-opt-02` hit `ERR_ROUTE_VIOLATION` at `adf_table_edit` (table cell mode against heading-only scoped ADF).

## Blocking Defect

**Scope resolver returns heading node, not heading section.**

`resolve_scope()` at `crates/atlassy-adf/src/lib.rs:48-95` uses `find_heading_paths()` to locate a heading by text match, then extracts just the heading node via `pointer_get()`. The returned `scoped_adf` contains only the heading node (e.g., 88 bytes), not the section (heading + subsequent sibling content until the next heading).

Manifest `target_path` values like `/content/1/content/0/text` point to paragraphs after the heading in the full-page ADF. These paths do not exist in the heading-only scoped ADF.

Why the scoped fetch spike did not catch this: spike runs used `--force-verify-fail` and `--mode no-op`, so the pipeline never reached the `patch` state where `target_path` is resolved against the scoped ADF.

## Gate Check Results

| Gate | Pass | Notes |
|------|------|-------|
| `telemetry_complete` | Yes | All 18 summaries include required KPI fields |
| `provenance_complete` | Yes | All outputs include valid provenance |
| `paired_matrix_complete` | **No** | No optimized runs completed — 0/9 pairs formed |
| `retry_policy` | Yes | No run exceeded one scoped retry |
| `drift_resolved` | Yes | Live-vs-stub parity checks resolved |
| `scenario_coverage` | Yes | S-001, S-010 both observed |
| `safety_gates` | Yes | No locked-node, out-of-scope, or table-shape violations |

## KPI Results

KPI section is `null` — the reporting pipeline requires `paired_matrix_complete` to compute KPI aggregates. Since no pairs formed, no KPI values were calculated.

Partial observations from baseline-only data:
- Baseline `edit_success_rate`: 100% (9/9).
- Baseline median `publish_latency`: ~1716ms (well within 3000ms target).
- Baseline `structural_preservation`: 100% (no locked-node mutations).
- Baseline `conflict_rate`: 0% (no retries).

## Notes

- `git_dirty: true` because the manifest (`qa/manifests/kpi-revalidation-batch.json`) and inventory updates were uncommitted at batch execution time.
- No `decision.packet.json` was generated — the batch report serves as the decision output.
- Batch execution order: C runs first, then A, then B (grouped by page_id).
- All 3 pages advanced to version 6 from baseline publishes, which means the page content has been modified 3 times from the seeded state. A re-run after fixing the scope resolver will need updated `target_path` values.
