# KPI and Experiments (v1)

## Objective

Define a reproducible measurement protocol that validates v1 goals for token efficiency, retrieval scope reduction, and formatting fidelity without relaxing safety gates.

## Scope and Alignment

- Dataset, edit patterns, and targets align with `08-poc-scope.md`.
- Route and safety constraints align with `06-decisions-and-defaults.md` and `09-ai-contract-spec.md`.
- Comparisons are baseline vs optimized for the same page and edit intent.

## KPI Definitions

### `tokens_per_successful_update`

- Definition: total model tokens consumed for runs that end with successful publish.
- Formula: `sum(total_tokens for successful runs) / count(successful runs)`.
- Unit: tokens per successful update.
- Success direction: lower is better.
- Target (v1 PoC): 40-60% reduction vs baseline median.

### `full_page_retrieval_rate`

- Definition: share of runs requiring full-page body retrieval.
- Formula: `count(runs with scope_resolution_failed=true or full_page_fetch=true) / count(all runs)`.
- Unit: percentage.
- Success direction: lower is better.
- Target (v1 PoC): 60-80% reduction vs baseline.

### `retry_conflict_token_waste`

- Definition: tokens spent on conflict retry paths only.
- Formula: `sum(tokens consumed after first conflict detection and before final publish/fail)`.
- Unit: tokens per run and aggregate tokens per experiment.
- Success direction: lower is better.
- Target (v1 PoC): bounded by one scoped retry policy; no unbounded retry loops.

### `formatting_fidelity_pass_rate`

- Definition: share of runs passing all verifier checks with no locked-node mutation.
- Formula: `count(verify_result=pass and locked_node_mutation=false) / count(all runs)`.
- Unit: percentage.
- Success direction: higher is better.
- Target (v1 PoC): non-regressive vs baseline.

### `publish_latency`

- Definition: elapsed wall-clock time from request start to publish result.
- Formula: `publish_end_timestamp - request_start_timestamp`.
- Unit: milliseconds.
- Success direction: lower is better.
- Target (v1 PoC): non-regressive vs baseline at median and p90.

## Experiment Design

### Design Type

- Paired A/B comparison with matched edit intent.
- `A` = baseline flow.
- `B` = optimized v1 flow (`fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> merge_candidates -> patch -> verify -> publish`).

### Run Matrix

- Patterns: A, B, C from `08-poc-scope.md`.
- Pages: baseline 5-page sample from `xilinx-wiki.atlassian.net` space `A`.
- Dataset provenance: baseline page profile and payload evidence are documented in `ideas/2026-03-confluence-adf-markdown-size-evidence.md`.
- Minimum runs: 3 runs per `(page_id, pattern, flow)` pair.
- Randomization: alternate A/B order per pair to reduce order bias.
- Retry policy: enforce one scoped rebase retry max for both flows.

### Controlled Variables

- Same page version window and same edit intent text per paired run.
- Same model family and comparable inference settings.
- Same verifier and publish gates.
- No expansion of v1 table or structural editing scope during PoC.

## Instrumentation Contract

Each run must emit one record with at least:

- `request_id`, `run_id`, `flow` (`baseline|optimized`), `pipeline_version`.
- `git_commit_sha` (full 40-character SHA for the build under test).
- `git_dirty` (boolean working-tree cleanliness marker at run start).
- `page_id`, `pattern` (`A|B|C`), `edit_intent_hash`.
- `scope_selectors`, `scope_resolution_failed`, `full_page_fetch`.
- `state_token_usage` map keyed by pipeline state.
- `total_tokens`, `retry_count`, `retry_tokens`.
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
- absolute delta and relative delta (`optimized vs baseline`).
- per-pattern breakdown (A/B/C) and aggregate breakdown.

Outlier handling:

- Keep all runs in primary analysis.
- Add a secondary view excluding runs with confirmed external-service incidents.
- Never exclude verifier failures caused by route or scope violations.

## Pass/Fail Rules

PoC passes when all are true:

- `tokens_per_successful_update` meets 40-60% reduction target vs baseline median.
- `full_page_retrieval_rate` meets 60-80% reduction target.
- `formatting_fidelity_pass_rate` is non-regressive.
- `publish_latency` is non-regressive at median and p90.
- No locked-node mutation is observed.
- No run exceeds one scoped conflict retry.

If any condition fails:

- Mark PoC as `iterate` if fixable without changing v1 scope.
- Mark PoC as `stop` if failures require relaxing core v1 safety constraints.

## Reporting

### Per-Run Summary

- `run_id`, `flow`, `page_id`, `pattern`, `success|fail`.
- `total_tokens`, `retry_tokens`, `full_page_fetch`, `latency_ms`.
- `verify_result`, `publish_result`, `error_codes[]`.

### Aggregate Report

- KPI table with baseline, optimized, delta, target, pass/fail.
- Pattern-level section (A/B/C) with notable regressions.
- Top outlier runs and suspected root causes.
- Recommendation: `go`, `iterate`, or `stop` with rationale.
- Provenance stamp with `git_commit_sha`, `git_dirty`, and `pipeline_version`.

## Current Checkpoint Snapshot

- Latest readiness recommendation: `iterate`.
- Primary blockers: `tokens_per_successful_update` and `full_page_retrieval_rate` target misses.
- KPI baseline: stub-backed execution. Live sandbox validation confirms runtime correctness and lifecycle behavior but does not yet include paired KPI revalidation runs.
- Live evidence: `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/` (runtime and lifecycle validation, not KPI-focused).

## Exit and Decision Update Workflow

- Publish experiment report with attached artifact index.
- Update `06-decisions-and-defaults.md` with measured outcomes and any default changes.
- If all gates pass, mark PoC execution complete in `03-phased-roadmap.md`.
- If gates fail, open follow-up items under `ideas/` or update phase sequencing.

## Threats to Validity

- Small dataset may underrepresent enterprise page complexity.
- Derived markdown comparisons may hide conversion-pipeline bias.
- External Confluence latency variance can affect publish latency metrics.
- Prompt drift across repeated runs can inflate variance without strict intent hashing.
