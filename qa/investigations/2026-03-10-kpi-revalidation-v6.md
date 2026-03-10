# 2026-03-10 KPI Revalidation v6 Investigation

## Provenance

- `git_commit_sha`: `217a942383a6c8784d4c20b65377e990a1db0422`
- `git_dirty`: `true`
- `pipeline_version`: `v1`
- `runtime_mode`: `live`
- Evidence bundle: `qa/evidence/2026-03-10-kpi-revalidation-v6/`
- Source run root: `qa/runs/full-steps-1-8-20260310T210423Z/artifacts/`
- Baseline for delta checks: `qa/evidence/2026-03-08-kpi-revalidation-v5/`
- Manifest: `qa/manifests/kpi-revalidation-v6-auto-discovery.json`

## Objective

Run full QA execution (Steps 1-8) on fresh KPI pages and determine whether readiness exits `iterate` after adding lifecycle attestation evidence.

## Environment and Setup

- QA env check: `make qa-check` -> 9 passed, 0 warnings, 0 failed.
- Parent sandbox page: `131207`.
- Fresh KPI pages created in this run:
  - P1: `1212417` (`KPI Experiment - Prose Rich v6 20260310T210423Z`)
  - P2: `1245185` (`KPI Experiment - Mixed Prose Table v6 20260310T210423Z`)
  - P3: `1245199` (`KPI Experiment - Locked Adjacent v6 20260310T210423Z`)
- Bootstrap matrix page created in Step 6: `1245213`.
- Fresh page content was seeded through Confluence REST API using valid ADF payloads (headings/tables/locked blocks) to keep setup reproducible without manual UI edits.

## Full QA Execution Summary

### Phase 1: Single-Page Smoke (Steps 1-7)

| Step | Run ID | Result |
|------|--------|--------|
| Step 1 (preflight) | `v6-preflight-001` | PASS -- expected verify failure (`ERR_SCHEMA_INVALID`) |
| Step 2b (single scoped fetch) | `v6-scoped-fetch-001` | PASS -- scoped fetch preserved (`scope_resolution_failed=false`, `full_page_fetch=false`) |
| Step 3 (live prose) | `v6-prose-001` | PASS -- `publish_result: "published"`, `new_version: 11` |
| Step 4 (live table, optional) | `v6-table-001` | PASS -- `publish_result: "published"`, `new_version: 4` |
| Step 5 (negative safety) | `v6-negative-prose-001` | PASS -- `ERR_SCHEMA_INVALID` at `md_assist_edit` |
| Step 6 (create subpage) | page `1245213` | PASS -- `page_version: 1` |
| Step 7a (empty, no flag) | `v6-bootstrap-required-001` | PASS -- `ERR_BOOTSTRAP_REQUIRED` |
| Step 7b (empty, with flag) | `v6-bootstrap-success-001` | PASS -- `bootstrap_applied: true`, published |
| Step 7c (non-empty, with flag) | `v6-bootstrap-invalid-001` | PASS -- `ERR_BOOTSTRAP_INVALID_STATE` |

### Phase 2: Scoped Fetch Spike + Auto-Discovery (Step 2b deep)

| Run ID | Scope | Result |
|--------|-------|--------|
| `v6-spike-fetch-p1-scoped` | `heading:Introduction` | PASS -- 73.97% reduction, scoped fetch preserved |
| `v6-spike-fetch-p2-scoped` | `heading:Data` | PASS -- 12.57% reduction, scoped fetch preserved |
| `v6-spike-fetch-p3-scoped` | `heading:Notes` | PASS -- 63.95% reduction, scoped fetch preserved |
| `v6-autodiscover-p1` | prose route | PASS -- discovered `/content/1/content/0/text` |
| `v6-autodiscover-p2-prose` | prose route | PASS -- discovered `/content/3/content/0/text` |
| `v6-autodiscover-p2-table` | table route | PASS -- discovered `/content/4/content/0/content/0/content/0/content/0/text` |
| `v6-autodiscover-p3` | prose route | PASS -- discovered `/content/4/content/0/text` |

### Phase 3: KPI Batch (18 runs)

- Batch command: `run-batch --manifest qa/manifests/kpi-revalidation-v6-auto-discovery.json --runtime-backend live`.
- Result: **18/18 succeeded**, **0 failed**, **9/9 pairs formed**.
- Batch status: `failed`, recommendation `iterate`.

## KPI Results

### Global

| KPI | Threshold | Result | Pass |
|-----|-----------|--------|------|
| `context_reduction_ratio` | Optimized median >= 70% | 64.18% | **FAIL** |
| `edit_success_rate` | > 95% | 100% | PASS |
| `structural_preservation` | 100% | 100% | PASS |
| `conflict_rate` | < 10% | 0% | PASS |
| `publish_latency` | Median < 3000ms, p90 <= baseline | median 1945ms, p90 2604ms vs baseline p90 2351ms | **FAIL** |

### Pattern Breakdown (optimized medians)

| Pattern | `context_reduction_ratio` | `publish_latency` | Notes |
|---------|---------------------------|-------------------|-------|
| A | 75.37% | 1945ms | Meets context target for pattern A |
| B | 11.96% | 2260ms | Low reduction remains driven by broad `heading:Data` scope |
| C | 64.18% | 1804ms | Stronger than B, still below 70% target |

## Delta vs v5 Baseline

### High-level

| Signal | v5 | v6 | Delta |
|--------|----|----|-------|
| Batch succeeded runs | 18/18 | 18/18 | no change |
| Recommendation | iterate | iterate | no change |
| Blocking condition | gate_7 lifecycle evidence | kpi_target_miss | improved gate coverage |
| Global `context_reduction_ratio` optimized median | 63.33% | 64.18% | +0.85 pts |
| Global `publish_latency` optimized median | 1414ms | 1945ms | +531ms |
| Global `publish_latency` optimized p90 | 1597ms | 2604ms | +1007ms |

### Signature parity checks

- Smoke/lifecycle deterministic signatures remained stable for expected fail/pass paths.
- No new safety error class appeared in smoke or batch diagnostics.
- Lifecycle readiness gate moved from fail to pass once attestation evidence was provided.

## Readiness Result

- `run-readiness --verify-replay` -> `recommendation: iterate`.
- Replay verification passed.
- `gate_7_lifecycle_enablement_validation`: **PASS** (attestation evidence consumed).
- Blocking condition: `kpi_target_miss`.
- Blocking KPIs: `context_reduction_ratio`, `publish_latency` p90 regression vs baseline.

## Recommendation

**iterate**

### Rationale

1. Full smoke/lifecycle and 18-run KPI execution completed successfully on fresh pages.
2. Lifecycle gate-coverage issue is resolved via attestation-backed evidence path.
3. Global context reduction remains below target despite slight improvement.
4. Publish latency p90 regressed versus baseline, introducing a second KPI miss.

## Next Steps

1. Tighten Pattern B selector strategy (narrow `heading:Data` scope or split section content) to improve context reduction without sacrificing reliability.
2. Investigate publish-latency regression drivers (page-version growth, payload size variance, sequential update pressure during batch).
3. Re-run KPI batch after selector and latency mitigation changes using fresh pages and the v6 manifest structure.
