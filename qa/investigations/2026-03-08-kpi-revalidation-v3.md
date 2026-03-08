# 2026-03-08 KPI Revalidation v3 Investigation

## Provenance

- `git_commit_sha`: `959755ca323504d5cba8820cc990dc9753b7fceb`
- `git_dirty`: `true`
- `pipeline_version`: `v1`
- `runtime_mode`: `live`
- Evidence bundle: `qa/evidence/2026-03-08-kpi-revalidation-v3/`
- Manifest: `qa/manifests/kpi-revalidation-v3-auto-discovery.json`

## Objective

Run full QA execution (Steps 1-8) on fresh KPI pages to verify the v2 investigation defect is resolved:

- Auto-discovery must not select heading text for prose route.
- Optimized heading-scoped runs must not fall back to full-page fetch.

## Environment and Setup

- QA env check: `make qa-check` -> 9 passed, 0 warnings, 0 failed.
- Fresh KPI pages created under parent `131207`:
  - P1: `98465` (`KPI Experiment - Prose Rich v3`)
  - P2: `327805` (`KPI Experiment - Mixed Prose Table v3`)
  - P3: `131294` (`KPI Experiment - Locked Adjacent v3`)
- Bootstrap matrix page created: `327819`.

## Full QA Execution Summary

### Phase 1: Single-Page Smoke (Steps 1, 3, 5, 6, 7)

| Step | Run ID | Result |
|------|--------|--------|
| Step 1 (preflight) | `v3-preflight-001` | PASS -- expected verify failure (`ERR_SCHEMA_INVALID`) |
| Step 3 (live prose) | `v3-prose-001` | PASS -- `publish_result: "published"`, `new_version: 9` |
| Step 5 (negative safety) | `v3-negative-prose-001` | PASS -- `ERR_SCHEMA_INVALID` at `md_assist_edit` |
| Step 6 (create subpage) | page `327819` | PASS -- `page_version: 1` |
| Step 7a (empty, no flag) | `v3-bootstrap-required-001` | PASS -- `ERR_BOOTSTRAP_REQUIRED` |
| Step 7b (empty, with flag) | `v3-bootstrap-success-001` | PASS -- `bootstrap_applied: true`, published |
| Step 7c (non-empty, with flag) | `v3-bootstrap-invalid-001` | PASS -- `ERR_BOOTSTRAP_INVALID_STATE` |

### Phase 2: Scoped Fetch Spike + Auto-Discovery (Step 2b)

| Run ID | Scope | Result |
|--------|-------|--------|
| `v3-spike-fetch-p1-scoped` | `heading:Introduction` | PASS -- 63.2% reduction, scoped fetch preserved |
| `v3-spike-fetch-p2-scoped` | `heading:Data` | PASS -- 12.6% reduction, scoped fetch preserved |
| `v3-spike-fetch-p3-scoped` | `heading:Notes` | PASS -- 62.9% reduction, scoped fetch preserved |
| `v3-autodiscover-p1` | prose route | PASS -- discovered `/content/1/content/0/text` |
| `v3-autodiscover-p2-prose` | prose route | PASS -- discovered `/content/3/content/0/text` |
| `v3-autodiscover-p2-table` | table route | PASS -- discovered `/content/4/content/0/content/0/content/0/content/0/text` |
| `v3-autodiscover-p3` | prose route | PASS -- discovered `/content/4/content/0/text` |

Additional selector gate checks (`v3-scopecheck-*`) passed for all headings used by the batch manifest (`Introduction`, `Details`, `Summary`, `Overview`, `Data`, `Context`, `Notes`, `References`) with:

- `scope_resolution_failed: false`
- `full_page_fetch: false`

### Phase 3: KPI Batch (18 runs)

- Batch command: `run-batch --manifest qa/manifests/kpi-revalidation-v3-auto-discovery.json --runtime-backend live`.
- Result: **18/18 succeeded**, **0 failed**, **9/9 pairs formed**.
- Batch status: `failed` (KPI gate misses), recommendation `iterate`.

## Defect Revalidation Outcome

### Prior defect signature (v2)

In v2, `*-opt-01` runs fell back to full-page fetch after baseline runs overwrote heading text.

### v3 result

Defect signature is absent:

- `kpi-v3-a-opt-01`: discovered `/content/1/content/0/text`, `scope_resolution_failed: false`, `full_page_fetch: false`
- `kpi-v3-b-opt-01`: discovered `/content/1/content/0/text`, `scope_resolution_failed: false`, `full_page_fetch: false`
- `kpi-v3-c-opt-01`: discovered `/content/1/content/0/text`, `scope_resolution_failed: false`, `full_page_fetch: false`

This confirms prose auto-discovery now resolves body text in scoped sections rather than heading anchor text for these pages.

## KPI Results

### Global

| KPI | Threshold | Result | Pass |
|-----|-----------|--------|------|
| `context_reduction_ratio` | Optimized median >= 70% | 63.84% | **FAIL** |
| `edit_success_rate` | > 95% | 100% | PASS |
| `structural_preservation` | 100% | 100% | PASS |
| `conflict_rate` | < 10% | 0% | PASS |
| `publish_latency` | Median < 3000ms, p90 non-regressive | median 1745ms, p90 1902ms vs baseline p90 1897ms | **FAIL** (p90 +5ms) |

### Pattern Breakdown (optimized medians)

| Pattern | `context_reduction_ratio` | `publish_latency` | Notes |
|---------|---------------------------|-------------------|-------|
| A | 64.97% | 1682ms | Good reduction, below 70% target median |
| B | 11.96% | 1802ms | Low reduction driven by `heading:Data` scope including large table |
| C | 63.84% | 1724ms | Strong scoped behavior, still below 70% target |

## Readiness Result

- `run-readiness --verify-replay` -> `recommendation: iterate`.
- Blocking condition: `gate_7_lifecycle_enablement_validation`.
- Blocking reason: lifecycle evidence is not represented in batch-manifest evidence structure (even though smoke lifecycle checks were executed in Phase 1).

## Recommendation

**iterate**

### Rationale

1. The investigation defect from v2 is resolved in live execution (no optimized scope fallback signature).
2. KPI targets still miss on:
   - `context_reduction_ratio` global optimized median (63.84% < 70%)
   - `publish_latency` p90 non-regression (+5ms over baseline)
3. Readiness remains blocked by lifecycle evidence packaging (`gate_7`).

## Next Steps

1. Improve pattern B scope strategy (section includes table-heavy content; consider narrower selector strategy if compatible with v1 constraints).
2. Decide readiness policy for lifecycle evidence ingestion (include explicit lifecycle runs in normalized manifest/report chain or adjust gate contract).
3. Re-run KPI batch after scope-strategy/gate-policy updates.
