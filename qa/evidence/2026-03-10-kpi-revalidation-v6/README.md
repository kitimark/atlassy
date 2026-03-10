# 2026-03-10 KPI Revalidation v6 Evidence Bundle

Full QA Steps 1-8 execution on fresh KPI pages with live runtime, including lifecycle attestation wiring for Gate 7 readiness evidence.

## Provenance

- `git_commit_sha`: `217a942383a6c8784d4c20b65377e990a1db0422`
- `git_dirty`: `true`
- `runtime_mode`: `live`
- `pipeline_version`: `v1`
- Parent sandbox page: `131207`
- KPI pages: P1=`1212417`, P2=`1245185`, P3=`1245199`
- Bootstrap matrix page created in this run: `1245213`
- Manifest: `qa/manifests/kpi-revalidation-v6-auto-discovery.json`
- Source artifact root: `qa/runs/full-steps-1-8-20260310T210423Z/artifacts/`
- Delta baseline: `qa/evidence/2026-03-08-kpi-revalidation-v5/`

## Bundle Contents

- `runs/`: smoke, scoped-spike, auto-discovery validation, lifecycle, and 18-run KPI batch artifacts copied from `qa/runs/full-steps-1-8-20260310T210423Z/artifacts/`.
- `report.json`: batch KPI report.
- `decision.packet.json`: batch recommendation packet.
- `readiness.checklist.json`: readiness gate checklist.
- `runbook.bundle.json`: readiness runbook packet.
- `manifest.normalized.json`: normalized manifest captured by `run-batch`.
- `artifact-index.json`: per-run artifact index emitted by batch execution.
- `attestations.json`: lifecycle readiness attestation consumed by Gate 7.

## Smoke and Scope Validation

- `v6-preflight-001`: expected verify failure (`ERR_SCHEMA_INVALID`), no publish.
- `v6-scoped-fetch-001`: scoped fetch preserved on P1 (`scope_resolution_failed: false`, `full_page_fetch: false`, reduction `73.97%`).
- `v6-prose-001`: scoped prose publish succeeded on primary page (`publish_result: "published"`).
- `v6-table-001`: optional table smoke publish succeeded on P2 (`publish_result: "published"`).
- `v6-negative-prose-001`: expected deterministic fail (`ERR_SCHEMA_INVALID`) at `md_assist_edit`.
- `v6-bootstrap-required-001`: expected `ERR_BOOTSTRAP_REQUIRED`.
- `v6-bootstrap-success-001`: bootstrap flagged run succeeded and published (`bootstrap_applied: true`).
- `v6-bootstrap-invalid-001`: expected `ERR_BOOTSTRAP_INVALID_STATE`.

## Scoped Spike and Auto-Discovery

- Scoped spike pass checks on all fresh pages (`scope_resolution_failed: false`, `full_page_fetch: false`):
  - P1 (`heading:Introduction`): `321 / 1233` bytes (`73.97%` reduction).
  - P2 (`heading:Data`): `1662 / 1901` bytes (`12.57%` reduction).
  - P3 (`heading:Notes`): `363 / 1007` bytes (`63.95%` reduction).
- Auto-discovery checks returned non-null paths:
  - `v6-autodiscover-p1`: `/content/1/content/0/text`
  - `v6-autodiscover-p2-prose`: `/content/3/content/0/text`
  - `v6-autodiscover-p2-table`: `/content/4/content/0/content/0/content/0/content/0/text`
  - `v6-autodiscover-p3`: `/content/4/content/0/text`

## Batch Summary

- Total runs: 18 (9 baseline, 9 optimized)
- Succeeded: **18/18**
- Failed: **0/18**
- Pair matrix: **9/9 complete**
- Recommendation: **iterate**

## Key KPI Results (Global)

| KPI | Threshold | Result | Pass |
|-----|-----------|--------|------|
| `context_reduction_ratio` | Optimized median >= 70% | 64.18% | **FAIL** |
| `edit_success_rate` | > 95% | 100% | PASS |
| `structural_preservation` | 100% | 100% | PASS |
| `conflict_rate` | < 10% | 0% | PASS |
| `publish_latency` | Median < 3000ms, p90 <= baseline | Median 1945ms, p90 2604ms (baseline p90 2351ms) | **FAIL** |

## Readiness Notes

- `run-readiness --verify-replay` produced recommendation `iterate` and replay verification passed.
- `gate_7_lifecycle_enablement_validation` passed using `artifacts/batch/attestations.json` evidence.
- Current blocking condition is `kpi_target_miss` (not gate coverage).
- Blocking KPI misses: `context_reduction_ratio` and `publish_latency`.

## Delta vs v5

| Signal | v5 | v6 | Delta |
|--------|----|----|-------|
| Batch succeeded runs | 18/18 | 18/18 | no change |
| Recommendation | iterate | iterate | no change |
| Blocking condition | gate_7 lifecycle evidence | kpi_target_miss | improved gate coverage, KPI now primary blocker |
| Global `context_reduction_ratio` optimized median | 63.33% | 64.18% | +0.85 pts |
| Global `publish_latency` optimized median | 1414ms | 1945ms | +531ms |
| Global `publish_latency` optimized p90 | 1597ms | 2604ms | +1007ms |
