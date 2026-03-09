# 2026-03-08 KPI Revalidation v5 Evidence Bundle

Full QA Steps 1-8 rerun on current HEAD to compare against the v4 baseline after recent CLI/pipeline refactors.

## Provenance

- `git_commit_sha`: `9c288f32ae3b646ec115150e6d0b42aef27df9b3`
- `git_dirty`: `true`
- `runtime_mode`: `live`
- `pipeline_version`: `v1`
- Parent sandbox page: `131207`
- KPI pages: P1=`131373`, P2=`131387`, P3=`327877`
- Bootstrap matrix page created in this run: `131438`
- Manifest: `qa/manifests/kpi-revalidation-v4-auto-discovery.json`
- Source artifact root: `qa/runs/full-steps-1-8-20260308T234657Z/artifacts/`
- Delta baseline: `qa/evidence/2026-03-08-kpi-revalidation-v4/`

## Bundle Contents

- `runs/`: smoke, scoped-spike, auto-discovery validation, lifecycle, and 18-run KPI batch artifacts copied from `qa/runs/full-steps-1-8-20260308T234657Z/artifacts/`.
- `report.json`: batch KPI report.
- `decision.packet.json`: batch recommendation packet.
- `readiness.checklist.json`: readiness gate checklist.
- `runbook.bundle.json`: readiness runbook packet.
- `manifest.normalized.json`: normalized manifest captured by `run-batch`.
- `artifact-index.json`: per-run artifact index emitted by batch execution.

## Smoke and Scope Validation

- Smoke/lifecycle signatures matched expected outcomes:
  - `live-preflight-001` failed at `verify` with `ERR_SCHEMA_INVALID` (expected no publish).
  - `live-scoped-fetch-001` remained scoped (`scope_resolution_failed: false`, `full_page_fetch: false`).
  - `live-prose-001` published successfully.
  - `live-negative-prose-boundary-001` failed with `ERR_SCHEMA_INVALID` at `md_assist_edit` (expected).
  - `live-bootstrap-required-001` failed with `ERR_BOOTSTRAP_REQUIRED`.
  - `live-bootstrap-success-001` published with `bootstrap_applied: true`.
  - `live-bootstrap-invalid-001` failed with `ERR_BOOTSTRAP_INVALID_STATE`.
- Scoped spike checks passed on P1/P2/P3 with `scope_resolution_failed: false` and `full_page_fetch: false`:
  - P1 (`heading:Introduction`): reduction 64.74% (`433`/`1228` bytes).
  - P2 (`heading:Data`): reduction 11.59% (`1655`/`1872` bytes).
  - P3 (`heading:Notes`): reduction 63.01% (`344`/`930` bytes).
- Auto-discovery checks passed with non-null `discovered_target_path` values:
  - `spike-autodiscover-p1`: `/content/1/content/0/text`
  - `spike-autodiscover-p2-table`: `/content/4/content/0/content/0/content/0/content/0/text`
  - `spike-autodiscover-p3`: `/content/4/content/0/text`
- Optional Step 4 table smoke on the primary sandbox page was not executed in this run.

## Batch Summary

- Total runs: 18 (9 baseline, 9 optimized)
- Succeeded: **18/18**
- Failed: **0/18**
- Pair matrix: **9/9 complete**
- Recommendation: **iterate**

## Key KPI Results (Global)

| KPI | Threshold | Result | Pass |
|-----|-----------|--------|------|
| `context_reduction_ratio` | Optimized median >= 70% | 63.33% | **FAIL** |
| `edit_success_rate` | > 95% | 100% | PASS |
| `structural_preservation` | 100% | 100% | PASS |
| `conflict_rate` | < 10% | 0% | PASS |
| `publish_latency` | Median < 3000ms, p90 <= baseline | Median 1414ms, p90 1597ms (baseline p90 1973ms) | PASS |

## Delta vs v4

| Signal | v4 | v5 | Delta |
|--------|----|----|-------|
| Batch succeeded runs | 18/18 | 18/18 | no change |
| Recommendation | iterate | iterate | no change |
| Global `context_reduction_ratio` optimized median | 63.33% | 63.33% | no change |
| Global `publish_latency` optimized median | 1755ms | 1414ms | **-341ms** |
| Blocking readiness gate | gate_7 | gate_7 | no change |

## Readiness Notes

- `run-readiness --verify-replay` produced recommendation `iterate`.
- Blocking condition remains `gate_7_lifecycle_enablement_validation`.
- Blocking reason remains unchanged: lifecycle evidence is not encoded in the batch-manifest evidence chain.
- KPI miss remains on global `context_reduction_ratio` optimized median.
