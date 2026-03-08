# 2026-03-08 KPI Revalidation v3 Evidence Bundle

Third KPI revalidation batch using fresh sandbox pages and auto-discovery manifest after the heading-exclusion fix landed in `discover_target_path()`.

## Provenance

- `git_commit_sha`: `959755ca323504d5cba8820cc990dc9753b7fceb`
- `git_dirty`: `true`
- `runtime_mode`: `live`
- `pipeline_version`: `v1`
- Parent sandbox page: `131207`
- KPI pages: P1=`98465`, P2=`327805`, P3=`131294`
- Bootstrap test page: `327819`
- Manifest: `qa/manifests/kpi-revalidation-v3-auto-discovery.json`

## Bundle Contents

- `runs/`: all smoke, scope-spike, auto-discovery validation, and batch run artifacts copied from `artifacts/full-qa-20260308T132959Z-v3/artifacts/`.
- `report.json`: batch KPI report.
- `decision.packet.json`: batch recommendation packet.
- `readiness.checklist.json`: readiness gate checklist.
- `runbook.bundle.json`: readiness runbook packet.
- `manifest.normalized.json`: normalized batch manifest captured by `run-batch`.
- `artifact-index.json`: per-run artifact index emitted by batch execution.

## Smoke and Scope Validation

- Smoke sequence (Steps 1, 3, 5, 6, 7) passed expected outcomes:
  - `v3-preflight-001` failed at `verify` with `ERR_SCHEMA_INVALID` (expected no publish).
  - `v3-prose-001` published successfully.
  - `v3-negative-prose-001` failed with `ERR_SCHEMA_INVALID` at `md_assist_edit` (expected).
  - `v3-bootstrap-required-001` failed with `ERR_BOOTSTRAP_REQUIRED`.
  - `v3-bootstrap-success-001` published with `bootstrap_applied: true`.
  - `v3-bootstrap-invalid-001` failed with `ERR_BOOTSTRAP_INVALID_STATE`.
- Scoped spike checks passed on P1/P2/P3 with `scope_resolution_failed: false` and `full_page_fetch: false`.
- Auto-discovery checks passed for prose/table routes (`v3-autodiscover-*`) with non-null `discovered_target_path` values.

## Batch Summary

- Total runs: 18 (9 baseline, 9 optimized)
- Succeeded: **18/18**
- Failed: **0/18**
- Pair matrix: **9/9 complete**
- Recommendation: **iterate**

## Key KPI Results (Global)

| KPI | Threshold | Result | Pass |
|-----|-----------|--------|------|
| `context_reduction_ratio` | Optimized median >= 70% | 63.84% | **FAIL** |
| `edit_success_rate` | > 95% | 100% | PASS |
| `structural_preservation` | 100% | 100% | PASS |
| `conflict_rate` | < 10% | 0% | PASS |
| `publish_latency` | Median < 3000ms, p90 <= baseline | Median 1745ms, p90 1902ms (baseline p90 1897ms) | **FAIL** (p90 +5ms) |

## Investigation Signal

- The v2 heading-overwrite defect signature is no longer present.
- `kpi-v3-a-opt-01`, `kpi-v3-b-opt-01`, and `kpi-v3-c-opt-01` all resolved scoped body paths (not heading text), with `scope_resolution_failed: false` and non-zero context reduction.
- Remaining KPI miss is dominated by pattern B scope size (`heading:Data` includes large table content), not selector fallback.

## Readiness Notes

- `run-readiness --verify-replay` produced recommendation `iterate`.
- Blocking condition: `gate_7_lifecycle_enablement_validation` (lifecycle evidence not encoded in batch manifest structure).
- KPI misses also listed in readiness rationale: `context_reduction_ratio`, `publish_latency` p90 non-regression.
