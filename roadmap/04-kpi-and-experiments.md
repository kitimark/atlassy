# KPI and Experiments

## Objective

Define reproducible measurement protocols for Atlassy capabilities: Foundation text-replacement PoC (Phases 0-5) and Structural operations (Phases 6-9) covering insert/edit/delete blocks across pages.

## Scope and Alignment

- Foundation PoC KPIs (Phases 0-5): dataset, edit patterns, and targets align with `08-poc-scope.md`.
- Structural KPIs (Phases 6-9): structural operation metrics defined under D-019.
- Route and safety constraints align with `06-decisions-and-defaults.md` and `09-ai-contract-spec.md`.
- Foundation comparisons are paired baseline vs optimized runs for the same page and edit intent.
- Structural comparisons measure operation correctness, schema validity, and precision.

## Foundation PoC KPI Definitions (Phases 0-5)

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
- `git_commit_sha` (full 40-character SHA embedded at compile time).
- `git_dirty` (boolean working-tree cleanliness marker embedded at build time).
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

## Foundation Checkpoint Snapshot

- Foundation readiness recommendation: `iterate` (superseded by Structural KPI framework per D-019).
- **2026-03-10 KPI revalidation v6 executed** (18 runs, 3 patterns, live runtime). Result: 18/18 runs succeeded, 9/9 pairs complete.
- Lifecycle readiness packaging gap is closed for Gate 7 via attestation evidence (`artifacts/batch/attestations.json`).
- Foundation KPI blockers: `context_reduction_ratio` optimized median `64.18%` (<70% threshold) and `publish_latency` p90 regression (`2604ms` optimized vs `2351ms` baseline).
- Pattern-level signal: A passes context target (`75.37%`), B remains primary reduction bottleneck (`11.96%`), C remains below global threshold (`64.18%`).
- The Pattern B selector issue is expected to be addressable through Phase 6 structural operations (inserting/deleting blocks on mixed-content pages) rather than narrower scoped fetching.
- Live evidence: `qa/evidence/2026-03-10-kpi-revalidation-v6/` (fresh-page run, Gate 7 pass, KPI-target miss).
- Prior evidence: `qa/evidence/2026-03-08-kpi-revalidation-v5/` and `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/`.

## Structural KPI Framework (Phases 6-9)

The Structural KPI framework replaces the Foundation framework as the primary measurement protocol (D-019). Foundation KPIs are retained as diagnostics but are no longer hard pass/fail gates.

### `operation_success_rate`

- Definition: share of insert/edit/delete operations that complete with successful publish.
- Formula: `count(publish_result=published) / count(all operations)`.
- Unit: percentage.
- Success direction: higher is better.
- Target: >95% for in-scope operations.
- Note: replaces Foundation `edit_success_rate` with expanded scope covering all operation types.

### `schema_validity_rate`

- Definition: share of post-mutation ADF documents that pass schema validation.
- Formula: `count(schema_valid=true) / count(all operations with insert or remove)`.
- Unit: percentage.
- Success direction: higher is better.
- Target: 100%.
- Note: new metric; not applicable to Foundation replace-only operations which preserve structure by construction.

### `operation_precision`

- Definition: share of operations where only the declared target blocks were affected.
- Formula: `count(changed_paths subset_of declared_operation_paths) / count(all operations)`.
- Unit: percentage.
- Success direction: higher is better.
- Target: 100%.
- Note: new metric; measures whether insert/delete operations have unintended side effects on adjacent blocks.

### `structural_integrity`

- Definition: share of operations where non-target structures are preserved, accounting for intentional structural changes declared in the operation manifest.
- Formula: `count(non_target_diff=empty) / count(all operations)`.
- Unit: percentage.
- Success direction: higher is better.
- Target: 100%.
- Note: evolves from Foundation `structural_preservation`. The key difference is that Structural operations intentionally change structure, so the metric must distinguish intentional changes from accidental side effects.

### `conflict_rate`

- Definition: share of operations that encounter at least one publish conflict.
- Formula: `count(retry_count > 0) / count(all operations)`.
- Unit: percentage.
- Success direction: lower is better.
- Target: <10%, with hard cap of one scoped retry per operation.
- Note: retained from Foundation; same definition and target.

### `publish_latency`

- Definition: elapsed wall-clock time from request start to publish result.
- Formula: `publish_end_timestamp - request_start_timestamp`.
- Unit: milliseconds.
- Success direction: lower is better.
- Target: median <3000 ms for scoped operations; p90 non-regressive vs replace-only baseline.
- Note: retained from Foundation; structural operations may have higher baseline latency due to schema validation.

### Demoted Foundation KPIs (diagnostic only)

- `context_reduction_ratio`: reported but no hard pass/fail threshold. Structural operations change the nature of what is fetched; context reduction is less meaningful when the goal is content modification rather than minimal-change editing.
- `scoped_section_tokens`: reported but no hard pass/fail threshold.

## Structural Pass/Fail Rules

Structural validation passes when all are true:

- `operation_success_rate` is >95% for in-scope operations.
- `schema_validity_rate` is 100% for all insert/delete operations.
- `operation_precision` is 100% for all operations.
- `structural_integrity` is 100% for all operations.
- `conflict_rate` is <10% and no operation exceeds one scoped conflict retry.
- `publish_latency` median for scoped operations is <3000 ms and p90 is non-regressive vs replace-only baseline.

If any condition fails:

- Mark phase as `iterate` if fixable within current phase scope.
- Mark phase as `stop` if failures require relaxing structural safety constraints.

## Structural Experiment Design

### Operation Matrix

- Pattern D: insert-only (add paragraphs, headings within scope).
- Pattern E: delete-only (remove paragraphs, headings within scope).
- Pattern F: mixed insert/edit/delete (structural modification + text replacement in same run).
- Pattern G: multi-page (coordinated operations across parent + child pages, Phase 8+).
- Patterns A/B/C from Foundation are retained for backward-compatibility regression testing.

### Controlled Variables

- Same page version window per operation.
- Same runtime backend, verifier, and publish gates.
- Post-mutation schema validation enabled for all insert/delete runs.
- Multi-operation batches use reverse-document-order processing.

## Exit and Decision Update Workflow

- Publish experiment report with attached artifact index.
- Update `06-decisions-and-defaults.md` with measured outcomes and any default changes.
- If all gates pass, mark phase execution complete in `03-phased-roadmap.md`.
- If gates fail, open follow-up items under `ideas/` or update phase sequencing.

## Threats to Validity

- Author-controlled sandbox pages may overestimate selector quality vs unmanaged enterprise content.
- Small datasets may underrepresent enterprise page complexity.
- External Confluence latency variance can affect publish latency metrics.
- Run intents without stable selector discipline can inflate variance.
- Insert/delete operations on small pages may not surface index-shift edge cases that appear on large pages.
- Post-mutation schema validation may mask latent structural issues if the ADF schema itself is incomplete.
