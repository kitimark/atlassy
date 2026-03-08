# 2026-03-08 KPI Revalidation v4 Investigation

## Provenance

- `git_commit_sha`: `18b7c633bf8a3ceae9185e19f4806ba1a61f06db`
- `git_dirty`: `true`
- `pipeline_version`: `v1`
- `runtime_mode`: `live`
- Evidence bundle: `qa/evidence/2026-03-08-kpi-revalidation-v4/`
- Manifest: `qa/manifests/kpi-revalidation-v4-auto-discovery.json`

## Objective

Run full QA execution (Steps 1-8) on fresh pages to revalidate scoped behavior and collect a new KPI/recommendation packet on current HEAD.

## Environment and Setup

- QA env check: `make qa-check` -> 9 passed, 0 warnings, 0 failed.
- Fresh KPI pages created under parent `131207`:
  - P1: `131373` (`KPI Experiment - Prose Rich v4 2026-03-08`)
  - P2: `131387` (`KPI Experiment - Mixed Prose Table v4 2026-03-08`)
  - P3: `327877` (`KPI Experiment - Locked Adjacent v4 2026-03-08`)
- Bootstrap matrix page created in Step 6: `327912`.

## Full QA Execution Summary

### Phase 1: Single-Page Smoke (Steps 1, 2b, 3, 4, 5, 6, 7)

| Step | Run ID | Result |
|------|--------|--------|
| Step 1 (preflight) | `v4-preflight-001` | PASS -- expected verify failure (`ERR_SCHEMA_INVALID`) |
| Step 2b (single scoped fetch) | `v4-scoped-fetch-001` | PASS -- scoped fetch preserved (`scope_resolution_failed=false`, `full_page_fetch=false`) |
| Step 3 (live prose) | `v4-prose-001` | PASS -- `publish_result: "published"`, `new_version: 3` |
| Step 4 (live table, optional) | `v4-table-001` | PASS -- `publish_result: "published"`, `new_version: 3` |
| Step 5 (negative safety) | `v4-negative-prose-001` | PASS -- `ERR_SCHEMA_INVALID` at `md_assist_edit` |
| Step 6 (create subpage) | page `327912` | PASS -- `page_version: 1` |
| Step 7a (empty, no flag) | `v4-bootstrap-required-001` | PASS -- `ERR_BOOTSTRAP_REQUIRED` |
| Step 7b (empty, with flag) | `v4-bootstrap-success-001` | PASS -- `bootstrap_applied: true`, published |
| Step 7c (non-empty, with flag) | `v4-bootstrap-invalid-001` | PASS -- `ERR_BOOTSTRAP_INVALID_STATE` |

### Phase 2: Scoped Fetch Spike + Auto-Discovery (Step 2b deep)

| Run ID | Scope | Result |
|--------|-------|--------|
| `v4-spike-fetch-p1-scoped` | `heading:Introduction` | PASS -- 64.2% reduction, scoped fetch preserved |
| `v4-spike-fetch-p2-scoped` | `heading:Data` | PASS -- 11.6% reduction, scoped fetch preserved |
| `v4-spike-fetch-p3-scoped` | `heading:Notes` | PASS -- 63.0% reduction, scoped fetch preserved |
| `v4-autodiscover-p1` | prose route | PASS -- discovered `/content/1/content/0/text` |
| `v4-autodiscover-p2-prose` | prose route | PASS -- discovered `/content/3/content/0/text` |
| `v4-autodiscover-p2-table` | table route | PASS -- discovered `/content/4/content/0/content/0/content/0/content/0/text` |
| `v4-autodiscover-p3` | prose route | PASS -- discovered `/content/4/content/0/text` |

### Phase 3: KPI Batch (18 runs)

- Batch command: `run-batch --manifest qa/manifests/kpi-revalidation-v4-auto-discovery.json --runtime-backend live`.
- Result: **18/18 succeeded**, **0 failed**, **9/9 pairs formed**.
- Batch status: `failed` (KPI target miss), recommendation `iterate`.

## KPI Results

### Global

| KPI | Threshold | Result | Pass |
|-----|-----------|--------|------|
| `context_reduction_ratio` | Optimized median >= 70% | 63.33% | **FAIL** |
| `edit_success_rate` | > 95% | 100% | PASS |
| `structural_preservation` | 100% | 100% | PASS |
| `conflict_rate` | < 10% | 0% | PASS |
| `publish_latency` | Median < 3000ms, p90 <= baseline | median 1755ms, p90 1894ms vs baseline p90 1942ms | PASS |

### Pattern Breakdown (optimized medians)

| Pattern | `context_reduction_ratio` | `publish_latency` | Notes |
|---------|---------------------------|-------------------|-------|
| A | 64.74% | 1652ms | Below 70% target median |
| B | 12.02% | 1755ms | Low reduction driven by broad `heading:Data` scope |
| C | 63.33% | 1761ms | Strong scoped behavior, still below 70% target |

## Readiness Result

- `run-readiness --verify-replay` -> `recommendation: iterate`.
- Blocking condition: `gate_7_lifecycle_enablement_validation`.
- Blocking reason: lifecycle evidence is not represented in batch-manifest evidence structure (smoke lifecycle checks passed separately).

## Recommendation

**iterate**

### Rationale

1. Full smoke/lifecycle and 18-run KPI execution completed successfully on fresh pages.
2. Safety and reliability KPIs passed (`edit_success_rate`, `structural_preservation`, `conflict_rate`, `publish_latency`).
3. Global context reduction target remains below threshold (63.33% < 70%).
4. Readiness remains blocked by gate-7 lifecycle evidence packaging requirements.

## Next Steps

1. Improve selector/scope strategy for pattern B to reduce scoped payload size.
2. Decide gate-7 evidence packaging approach (encode lifecycle runs into batch evidence chain or revise gate contract).
3. Re-run KPI batch after scope-strategy and/or readiness gate updates.
