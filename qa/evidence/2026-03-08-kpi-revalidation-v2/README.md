# 2026-03-08 KPI Revalidation v2 Evidence Bundle

Second KPI revalidation batch using auto-discovery manifests after fixing both blocking defects from v1: section extraction (`86cf652`) and scoped extraction filtering (`a611f58`).

## Provenance

- `git_commit_sha`: `888772c0d8eec5ba3e64840e300200e8f1a50e61`
- `git_dirty`: `false`
- `runtime_mode`: `live`
- `pipeline_version`: `v1`
- Sandbox pages: P1=`65934` (prose-rich), P2=`98323` (prose+table), P3=`131227` (locked-adjacent)
- Parent page: `131207`
- Manifest: `qa/manifests/kpi-revalidation-auto-discovery.example.json`

## Batch Summary

- Total runs: 18 (9 baseline, 9 optimized)
- Baseline succeeded: **9/9** (100%)
- Optimized succeeded: **9/9** (100%) -- up from 0/9 in v1
- Pairs formed: **9/9** (100%)
- Recommendation: **iterate**
- Blocking gates: `context_reduction_ratio`, `publish_latency` (p90 regression), `gate_7_lifecycle_enablement_validation`

## Included Runs

### Baseline Runs (9/9 success)

| Run ID | Page | Pattern | Latency | Version | Publish |
|--------|------|---------|---------|---------|---------|
| `kpi-v2-c-base-01` | P3 (131227) | C | 2302ms | 7 | published |
| `kpi-v2-c-base-02` | P3 (131227) | C | 1733ms | 9 | published |
| `kpi-v2-c-base-03` | P3 (131227) | C | 1527ms | 11 | published |
| `kpi-v2-a-base-01` | P1 (65934) | A | 1610ms | 7 | published |
| `kpi-v2-a-base-02` | P1 (65934) | A | 1806ms | 9 | published |
| `kpi-v2-a-base-03` | P1 (65934) | A | 2589ms | 11 | published |
| `kpi-v2-b-base-01` | P2 (98323) | B | 1647ms | 7 | published |
| `kpi-v2-b-base-02` | P2 (98323) | B | 1737ms | 9 | published |
| `kpi-v2-b-base-03` | P2 (98323) | B | 1817ms | 11 | published |

### Optimized Runs (9/9 success)

| Run ID | Page | Pattern | Scope Selector | Latency | Context Reduction | Scope Failed |
|--------|------|---------|----------------|---------|-------------------|--------------|
| `kpi-v2-c-opt-01` | P3 | C | `heading:Context` | 1743ms | 0.0% | **true** |
| `kpi-v2-c-opt-02` | P3 | C | `heading:Notes` | 1646ms | 68.1% | false |
| `kpi-v2-c-opt-03` | P3 | C | `heading:References` | 1746ms | 77.6% | false |
| `kpi-v2-a-opt-01` | P1 | A | `heading:Introduction` | 1665ms | 0.0% | **true** |
| `kpi-v2-a-opt-02` | P1 | A | `heading:Details` | 2115ms | 49.5% | false |
| `kpi-v2-a-opt-03` | P1 | A | `heading:Summary` | 2623ms | 81.8% | false |
| `kpi-v2-b-opt-01` | P2 | B | `heading:Overview` | 1538ms | 0.0% | **true** |
| `kpi-v2-b-opt-02` | P2 | B | `heading:Data` | 1739ms | 18.2% | false |
| `kpi-v2-b-opt-03` | P2 | B | `heading:Data` | 1594ms | 17.8% | false |

## Defect Found: Auto-Discovery Targets Heading Text Nodes

3/9 optimized runs (`*-opt-01` on each page) fell back to full-page fetch because their heading selectors no longer matched. Root cause: the preceding baseline run (`*-base-01`) used auto-discovery with full-page scope, which resolved to `/content/0/content/0/text` -- the first heading's text node. The baseline run overwrote the heading text (e.g., "Introduction" became "KPI-v2 baseline A1 prose update"), making the heading selector in the subsequent optimized run unmatchable.

The `discover_target_path` function treats heading text as a valid prose target because headings are in the `EDITABLE_PROSE_TYPES` whitelist. This is functionally correct (heading text IS editable) but creates a footgun when paired with heading-based scope selectors: modifying a heading destroys the selector's ability to find it.

Fix options:
- Prefer paragraph text nodes over heading text in auto-discovery.
- Exclude heading text nodes from prose discovery entirely.
- Add a `--exclude-headings-from-discovery` flag.

## KPI Results

| KPI | Threshold | Global | Pass |
|-----|-----------|--------|------|
| `context_reduction_ratio` | Optimized median >= 70% | 18.2% | **FAIL** |
| `edit_success_rate` | > 95% | 100% | PASS |
| `structural_preservation` | 100% | 100% | PASS |
| `conflict_rate` | < 10% | 0% | PASS |
| `publish_latency` | Median < 3000ms, p90 non-regressive | Median 1739ms, p90 2623ms | **FAIL** (p90) |

### Context Reduction Analysis

The global median of 18.2% is misleading. Excluding the 3 fallback runs (caused by the heading-overwrite defect):

| Pattern | Optimized Runs (scope OK) | Reduction Range |
|---------|---------------------------|-----------------|
| A | opt-02 (49.5%), opt-03 (81.8%) | 49-82% |
| B | opt-02 (18.2%), opt-03 (17.8%) | 17-18% |
| C | opt-02 (68.1%), opt-03 (77.6%) | 68-78% |

Pattern B shows low reduction because the "Data" heading section contains a large table (most of the page content). Patterns A and C demonstrate real 50-82% reduction when scope resolution succeeds.

### Publish Latency Analysis

The p90 regression (optimized 2623ms vs baseline 2589ms) is a 34ms difference driven by one outlier (`kpi-v2-a-opt-03` at 2623ms). Median latency is nearly identical (baseline 1737ms vs optimized 1739ms). This is likely network variance, not a systematic regression.

## Gate Check Results

| Gate | Pass | Notes |
|------|------|-------|
| `telemetry_complete` | Yes | All 18 summaries include required KPI fields |
| `provenance_complete` | Yes | `git_dirty: false` |
| `paired_matrix_complete` | Yes | 9/9 pairs formed (up from 0/9 in v1) |
| `retry_policy` | Yes | Zero retries across all 18 runs |
| `drift_resolved` | Yes | No material drift |
| `scenario_coverage` | Yes | S-001, S-010 both covered |
| `safety_gates` | Yes | Zero locked-node, out-of-scope, or table-shape violations |
| `gate_7_lifecycle` | No | Lifecycle evidence not in batch manifest |

## Notes

- Pages advanced to version 12 from interleaved baseline/optimized publishes.
- `gate_7_lifecycle_enablement_validation` fails because lifecycle runs (Steps 6-7) are not included in the batch manifest. These were validated separately in Phase 1 smoke testing.
- Batch execution order: C runs first, then A, then B (grouped by page_id).
