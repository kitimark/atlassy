# KPI and Experiments (v1)

## Objective

Define a reproducible measurement protocol for the CLI-first PoC that captures real-world value for AI-assisted editing, while preserving v1 safety defaults.

## Scope and Alignment

- Dataset, edit patterns, and targets align with `08-poc-scope.md`.
- Route and safety constraints align with `06-decisions-and-defaults.md` and `09-ai-contract-spec.md`.
- Comparisons are paired baseline vs optimized runs for the same page and edit intent.
- v1 remains CLI-first; metrics are selected to predict future MCP usage value.

## KPI Definitions

### `context_reduction_ratio`

- Definition: relative reduction from full-page payload to scoped payload per run.
- Formula: `1 - (scoped_adf_bytes / full_page_adf_bytes)`.
- Unit: ratio (reported as percentage).
- Success direction: higher is better.
- Target (v1 PoC): 70-90% for in-scope optimized runs.

### `scoped_section_tokens`

- Definition: estimated token footprint of scoped payload delivered to edit logic.
- Formula: `scoped_adf_bytes / 4` (byte-to-token estimator).
- Unit: estimated tokens per run.
- Success direction: lower is better.
- Target (v1 PoC): report median and p90 by pattern; no fixed global threshold.

### `edit_success_rate`

- Definition: share of runs that complete with successful publish.
- Formula: `count(publish_result=published) / count(all runs)`.
- Unit: percentage.
- Success direction: higher is better.
- Target (v1 PoC): >95% for in-scope patterns.

### `structural_preservation`

- Definition: share of runs passing verifier gates without locked-node or out-of-scope mutation.
- Formula: `count(verify_result=pass and locked_node_mutation=false and out_of_scope_mutation=false) / count(all runs)`.
- Unit: percentage.
- Success direction: higher is better.
- Target (v1 PoC): 100% on non-target structure for in-scope runs.

### `conflict_rate`

- Definition: share of runs that encounter at least one publish conflict.
- Formula: `count(retry_count > 0) / count(all runs)`.
- Unit: percentage.
- Success direction: lower is better.
- Target (v1 PoC): <10%, with hard cap of one scoped retry per run.

### `publish_latency`

- Definition: elapsed wall-clock time from request start to publish result.
- Formula: `publish_end_timestamp - request_start_timestamp`.
- Unit: milliseconds.
- Success direction: lower is better.
- Target (v1 PoC): median <3000 ms for scoped optimized runs; p90 non-regressive vs paired baseline.

## Experiment Design

### Design Type

- Paired A/B comparison with matched edit intent and target section.
- `A` baseline: same edit intent with empty `scope_selectors` (full-page fallback path).
- `B` optimized: same edit intent with explicit heading/block selectors.

### Run Matrix

- Patterns: A, B, C from `08-poc-scope.md`.
- Pages: controlled sandbox pages with planned structural variety (size, prose/table mix, locked structural adjacency).
- Optional reference pages: 5-page public seed from `xilinx-wiki.atlassian.net` space `A`.
- Minimum runs: 3 runs per `(page_id, pattern, flow)` pair.
- Randomization: alternate A/B order per pair to reduce order bias.
- Retry policy: enforce one scoped rebase retry max for both flows.

### Controlled Variables

- Same page version window and same edit intent text per paired run.
- Same target section semantics across baseline and optimized runs.
- Same runtime backend, verifier, and publish gates.
- No expansion of v1 table or structural editing scope during PoC.

## Instrumentation Contract

Each run must emit one record with at least:

- `request_id`, `run_id`, `flow` (`baseline|optimized`), `pipeline_version`.
- `git_commit_sha` (full 40-character SHA for the build under test).
- `git_dirty` (boolean working-tree cleanliness marker at run start).
- `page_id`, `pattern` (`A|B|C`), `edit_intent_hash`.
- `scope_selectors`, `scope_resolution_failed`, `full_page_fetch`.
- `full_page_adf_bytes`, `scoped_adf_bytes`, `context_reduction_ratio`.
- `patch_ops_bytes`, `retry_count`.
- `state_token_usage` map keyed by pipeline state (supporting telemetry).
- `verify_result`, `verify_error_codes[]`.
- `publish_result`, `publish_error_code?`, `new_version?`.
- `start_ts`, `verify_end_ts`, `publish_end_ts`, `latency_ms`.

Replay artifacts per run:

- `artifacts/<run_id>/<state>/state_input.json`
- `artifacts/<run_id>/<state>/state_output.json`
- `artifacts/<run_id>/<state>/diagnostics.json`
- `artifacts/<run_id>/summary.json`

## Analysis Method

For each KPI, compute:

- median, p90, min, max by flow.
- absolute delta and relative delta (`optimized vs baseline`) where applicable.
- per-pattern breakdown (A/B/C) and aggregate breakdown.

Additional required slices:

- per-page `context_reduction_ratio` distribution.
- fallback reason breakdown for `full_page_fetch=true` runs.

Outlier handling:

- Keep all runs in primary analysis.
- Add a secondary view excluding runs with confirmed external-service incidents.
- Never exclude verifier failures caused by route or scope violations.

## Pass/Fail Rules

PoC passes when all are true:

- `context_reduction_ratio` median for optimized flow is >=70% and trend supports 70-90% target band.
- `edit_success_rate` is >95% for in-scope runs.
- `structural_preservation` is 100% for in-scope runs.
- `conflict_rate` is <10% and no run exceeds one scoped conflict retry.
- `publish_latency` median for optimized scoped runs is <3000 ms and p90 is non-regressive vs baseline.
- No locked-node mutation is observed.

If any condition fails:

- Mark PoC as `iterate` if fixable without changing v1 scope.
- Mark PoC as `stop` if failures require relaxing core v1 safety constraints.

## Reporting

### Per-Run Summary

- `run_id`, `flow`, `page_id`, `pattern`, `success|fail`.
- `full_page_adf_bytes`, `scoped_adf_bytes`, `context_reduction_ratio`.
- `patch_ops_bytes`, `retry_count`, `latency_ms`.
- `verify_result`, `publish_result`, `error_codes[]`.

### Aggregate Report

- KPI table with baseline, optimized, delta, target, pass/fail.
- Pattern-level section (A/B/C) with notable regressions.
- Fallback reason section for scope misses.
- Top outlier runs and suspected root causes.
- Recommendation: `go`, `iterate`, or `stop` with rationale.
- Provenance stamp with `git_commit_sha`, `git_dirty`, and `pipeline_version`.

## Current Checkpoint Snapshot

- Latest readiness recommendation: `iterate`.
- **2026-03-08 KPI revalidation batch executed** (18 runs, 3 patterns, live runtime). Result: 9/9 baseline succeeded, 0/9 optimized succeeded.
- Blocking defect: scope resolver returns heading node only, not heading section. All optimized runs fail at `patch` with `ERR_SCHEMA_INVALID` because `target_path` references nodes outside the scoped ADF. See `qa/investigations/2026-03-08-kpi-revalidation.md`.
- Positive signals from batch: baseline pipeline fully validated (100% publish success, zero retries, median latency ~1716ms), scope resolution finds headings correctly (96% context reduction), safety gates held on all 18 runs.
- Next step: fix section extraction in `resolve_scope()` (`crates/atlassy-adf/src/lib.rs`), re-seed pages, re-run batch. See `ideas/2026-03-scope-resolution-quality.md`.
- Live evidence: `qa/evidence/2026-03-08-kpi-revalidation/` (KPI revalidation batch, blocking defect documented).
- Prior evidence: `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/` (runtime and lifecycle validation).

## Blocking Prerequisites for Re-Run

The following must be resolved before the KPI batch can be re-run with valid paired results:

- **Section extraction fix** (P0): `resolve_scope()` must return the heading section, not just the heading node. All 9 optimized runs in the 2026-03-08 batch failed because the scoped ADF contained only the heading (~88 bytes), making `target_path` references to subsequent paragraphs invalid. See `ideas/2026-03-scope-resolution-quality.md` (promoted).
- **Section extraction unit tests** (P0): regression safety net for the fix. Required cases: heading with trailing content, heading at end of array, adjacent headings (empty section), nested content, multi-selector merge.
- **`seed-page` CLI command** (P1): pages are at version 6 after baseline publishes. Re-seeding requires either manual Confluence UI editing or `curl` — both non-reproducible. The `seed-page` command enables automated, reproducible page setup. See `ideas/2026-03-raw-adf-page-seeding.md` (promoted).
- **Scoped pipeline integration test** (P1): at least one integration test with non-empty `scope_selectors` to prevent regression of the scoped pipeline path.

## Exit and Decision Update Workflow

- Publish experiment report with attached artifact index.
- Update `06-decisions-and-defaults.md` with measured outcomes and any default changes.
- If all gates pass, mark PoC execution complete in `03-phased-roadmap.md`.
- If gates fail, open follow-up items under `ideas/` or update phase sequencing.

## Threats to Validity

- Author-controlled sandbox pages may overestimate selector quality vs unmanaged enterprise content.
- Small datasets may underrepresent enterprise page complexity.
- External Confluence latency variance can affect publish latency metrics.
- Run intents without stable selector discipline can inflate variance.
