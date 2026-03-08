# 2026-03-08 KPI Revalidation v4 Evidence Bundle

Fourth KPI revalidation run on fresh sandbox pages and a fresh auto-discovery manifest after the leaf-crate modularization merge.

## Provenance

- `git_commit_sha`: `18b7c633bf8a3ceae9185e19f4806ba1a61f06db`
- `git_dirty`: `true`
- `runtime_mode`: `live`
- `pipeline_version`: `v1`
- Parent sandbox page: `131207`
- KPI pages: P1=`131373`, P2=`131387`, P3=`327877`
- Bootstrap matrix page: `327912`
- Manifest: `qa/manifests/kpi-revalidation-v4-auto-discovery.json`

## Bundle Contents

- `runs/`: smoke, scoped-spike, auto-discovery validation, and 18-run KPI batch artifacts copied from `artifacts/full-qa-20260308T181918Z-v4/artifacts/`.
- `report.json`: batch KPI report.
- `decision.packet.json`: batch recommendation packet.
- `readiness.checklist.json`: readiness gate checklist.
- `runbook.bundle.json`: readiness runbook packet.
- `manifest.normalized.json`: normalized batch manifest captured by `run-batch`.
- `artifact-index.json`: per-run artifact index emitted by batch execution.

## Smoke and Scope Validation

- Smoke/lifecycle sequence passed expected signatures:
  - `v4-preflight-001` failed at `verify` with `ERR_SCHEMA_INVALID` (expected no publish).
  - `v4-scoped-fetch-001` remained scoped (`scope_resolution_failed: false`, `full_page_fetch: false`).
  - `v4-prose-001` published successfully.
  - `v4-table-001` published successfully.
  - `v4-negative-prose-001` failed with `ERR_SCHEMA_INVALID` at `md_assist_edit` (expected).
  - `v4-bootstrap-required-001` failed with `ERR_BOOTSTRAP_REQUIRED`.
  - `v4-bootstrap-success-001` published with `bootstrap_applied: true`.
  - `v4-bootstrap-invalid-001` failed with `ERR_BOOTSTRAP_INVALID_STATE`.
- Scoped spike checks passed on P1/P2/P3 with `scope_resolution_failed: false` and `full_page_fetch: false`.
- Auto-discovery checks passed for prose/table routes (`v4-autodiscover-*`) with non-null `discovered_target_path` values.

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
| `publish_latency` | Median < 3000ms, p90 <= baseline | Median 1755ms, p90 1894ms (baseline p90 1942ms) | PASS |

## Readiness Notes

- `run-readiness --verify-replay` produced recommendation `iterate`.
- Blocking condition: `gate_7_lifecycle_enablement_validation`.
- Blocking reason: lifecycle evidence is not encoded in batch-manifest evidence structure.
- KPI miss remains on `context_reduction_ratio` global optimized median.
