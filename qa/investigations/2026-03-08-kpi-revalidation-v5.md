# 2026-03-08 KPI Revalidation v5 Investigation

## Provenance

- `git_commit_sha`: `9c288f32ae3b646ec115150e6d0b42aef27df9b3`
- `git_dirty`: `true`
- `pipeline_version`: `v1`
- `runtime_mode`: `live`
- Evidence bundle: `qa/evidence/2026-03-08-kpi-revalidation-v5/`
- Source run root: `qa/runs/full-steps-1-8-20260308T234657Z/artifacts/`
- Baseline for delta checks: `qa/evidence/2026-03-08-kpi-revalidation-v4/`
- Manifest: `qa/manifests/kpi-revalidation-v4-auto-discovery.json`

## Objective

Run full QA execution (Steps 1-8) on current HEAD and compare outcomes against the latest committed v4 QA execution.

## Environment and Setup

- QA env check: `make qa-check` -> 9 passed, 0 warnings, 0 failed.
- Parent sandbox page: `131207`.
- KPI pages reused from v4 inventory:
  - P1: `131373`
  - P2: `131387`
  - P3: `327877`
- Bootstrap matrix page created in Step 6: `131438`.

## Full QA Execution Summary

### Phase 1: Single-Page Smoke (Steps 1, 2b, 3, 5, 6, 7)

| Step | Run ID | Result |
|------|--------|--------|
| Step 1 (preflight) | `live-preflight-001` | PASS -- expected verify failure (`ERR_SCHEMA_INVALID`) |
| Step 2b (single scoped fetch) | `live-scoped-fetch-001` | PASS -- scoped fetch preserved (`scope_resolution_failed=false`, `full_page_fetch=false`) |
| Step 3 (live prose) | `live-prose-001` | PASS -- `publish_result: "published"`, `new_version: 10` |
| Step 4 (live table, optional) | not run | SKIPPED -- optional step on primary page |
| Step 5 (negative safety) | `live-negative-prose-boundary-001` | PASS -- `ERR_SCHEMA_INVALID` at `md_assist_edit` |
| Step 6 (create subpage) | page `131438` | PASS -- `page_version: 1` |
| Step 7a (empty, no flag) | `live-bootstrap-required-001` | PASS -- `ERR_BOOTSTRAP_REQUIRED` |
| Step 7b (empty, with flag) | `live-bootstrap-success-001` | PASS -- `bootstrap_applied: true`, published |
| Step 7c (non-empty, with flag) | `live-bootstrap-invalid-001` | PASS -- `ERR_BOOTSTRAP_INVALID_STATE` |

### Phase 2: Scoped Fetch Spike + Auto-Discovery (Step 2b deep)

| Run ID | Scope | Result |
|--------|-------|--------|
| `spike-fetch-p1-scoped` | `heading:Introduction` | PASS -- 64.74% reduction, scoped fetch preserved |
| `spike-fetch-p2-scoped` | `heading:Data` | PASS -- 11.59% reduction, scoped fetch preserved |
| `spike-fetch-p3-scoped` | `heading:Notes` | PASS -- 63.01% reduction, scoped fetch preserved |
| `spike-autodiscover-p1` | prose route | PASS -- discovered `/content/1/content/0/text` |
| `spike-autodiscover-p2-table` | table route | PASS -- discovered `/content/4/content/0/content/0/content/0/content/0/text` |
| `spike-autodiscover-p3` | prose route | PASS -- discovered `/content/4/content/0/text` |

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
| `publish_latency` | Median < 3000ms, p90 <= baseline | median 1414ms, p90 1597ms vs baseline p90 1973ms | PASS |

### Pattern Breakdown (optimized medians)

| Pattern | `context_reduction_ratio` | `publish_latency` | Notes |
|---------|---------------------------|-------------------|-------|
| A | 64.74% | 829ms | Below 70% target median |
| B | 12.02% | 1438ms | Low reduction driven by broad `heading:Data` scope |
| C | 63.33% | 1414ms | Strong scoped behavior, still below 70% target |

## Delta vs v4 Baseline

### High-level

| Signal | v4 | v5 | Delta |
|--------|----|----|-------|
| Batch succeeded runs | 18/18 | 18/18 | no change |
| Batch recommendation | iterate | iterate | no change |
| Global `context_reduction_ratio` optimized median | 63.33% | 63.33% | no change |
| Global `publish_latency` optimized median | 1755ms | 1414ms | **-341ms** |
| Blocking condition | `gate_7_lifecycle_enablement_validation` | `gate_7_lifecycle_enablement_validation` | no change |

### Signature parity checks

- Smoke/lifecycle signature parity matches v4 for:
  - preflight verify fail (`ERR_SCHEMA_INVALID`)
  - scoped fetch no full-page fallback
  - negative safety deterministic fail (`ERR_SCHEMA_INVALID`)
  - bootstrap required / success / invalid-state matrix
- No new failure class was observed in smoke/lifecycle or batch diagnostics.

## Readiness Result

- `run-readiness --verify-replay` -> `recommendation: iterate`.
- Blocking condition: `gate_7_lifecycle_enablement_validation`.
- Blocking reason: lifecycle evidence is not represented in the batch-manifest evidence structure.

## Recommendation

**iterate**

### Rationale

1. Full run completed with no regressions versus the v4 execution baseline.
2. Safety and reliability KPIs remained green (`edit_success_rate`, `structural_preservation`, `conflict_rate`, `publish_latency`).
3. Global context reduction target is still below threshold (63.33% < 70%).
4. Readiness remains blocked by unchanged gate-7 lifecycle evidence packaging requirements.

## Next Steps

1. Improve scope strategy for Pattern B (`heading:Data`) to increase context reduction while maintaining edit reliability.
2. Encode lifecycle smoke evidence into the decision-grade evidence chain or update gate-7 contract expectations.
3. Re-run KPI batch after scope-strategy and/or gate-7 evidence-chain updates.
