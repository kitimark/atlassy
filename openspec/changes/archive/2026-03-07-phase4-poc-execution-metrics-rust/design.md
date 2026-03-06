## Context

Phases 1-3 delivered v1 behavior for orchestration, prose assist, and table cell edits with deterministic safety checks. Phase 4 shifts focus from feature delivery to evidence generation: we must run reproducible paired experiments and produce decision-grade KPI outputs aligned with roadmap definitions.

Operational constraints are strict:
- Paired baseline vs optimized runs must share page and intent identity.
- Retry policy remains one scoped retry max.
- Safety gates remain hard blockers (locked-node mutation, out-of-scope edits, table shape drift).
- Evidence must be reproducible from stored artifacts and machine-readable summaries.

## Goals / Non-Goals

**Goals:**
- Build a deterministic run-matrix executor for Pattern A/B/C across baseline pages.
- Standardize per-run telemetry and artifact index output to support KPI aggregation.
- Produce aggregate reports with median/p90, deltas, target checks, and recommendation support.
- Add drift and scenario-coverage checks so PoC sign-off reflects both KPI and safety integrity.

**Non-Goals:**
- Expanding v1 editing scope (advanced table ops, structural edits, multi-page orchestration).
- Reworking pipeline state order or replacing existing verifier contracts.
- Building long-term analytics infrastructure beyond PoC decision needs.

## Decisions

1. PoC orchestration is manifest-driven from CLI.
   - Decision: add run-manifest input defining `(run_id, page_id, pattern, flow, edit_intent_hash)` with deterministic ordering and explicit baseline/optimized pairing.
   - Rationale: manifest-first execution avoids hidden scheduling variation and makes reruns reproducible.
   - Alternatives considered:
     - Ad hoc CLI loops: rejected due to weak reproducibility and difficult auditability.
     - DB-backed scheduling: rejected as too heavy for Phase 4 scope.

2. Telemetry is emitted at run-completion in a normalized schema.
   - Decision: persist a single run summary record with required KPI fields plus per-state token usage and verifier/publish outcomes, then index by run ID.
   - Rationale: one canonical summary per run simplifies aggregation and mismatch detection.
   - Alternatives considered:
     - Streaming-only metrics: rejected because post-hoc reproducibility is weaker.
     - State-fragment metrics only: rejected because KPI reporting needs a consolidated record.

3. KPI aggregation is deterministic and rule-based.
   - Decision: implement deterministic aggregators for median/p90 and delta calculations, including per-pattern and global rollups, with explicit pass/fail checks tied to roadmap thresholds.
   - Rationale: explicit rule code avoids manual interpretation drift and supports repeat decision packets.
   - Alternatives considered:
     - Manual spreadsheet analysis: rejected due to error risk and low repeatability.
     - Unbounded statistical modeling: rejected as unnecessary for v1 PoC decisions.

4. Drift validation combines stub scenario checks with live smoke parity gates.
   - Decision: require scheduled live probe outputs to be compared against stub expectations for key behaviors (scope miss, publish conflict, error payload classes) and gate reporting if unresolved drift exists.
   - Rationale: KPI results are only meaningful if behavior assumptions remain aligned with live Confluence.
   - Alternatives considered:
     - Stub-only validation: rejected due to behavior drift blind spots.
     - Live-only execution: rejected due to cost/flake and limited reproducibility.

5. Failure handling is fail-fast with explicit recommendation states.
   - Decision: if telemetry completeness, safety gates, or retry policy invariants are violated, batch status is marked failed and recommendation defaults to `iterate` or `stop` based on violation type.
   - Rationale: preserves decision quality and prevents KPI cherry-picking on invalid runs.
   - Alternatives considered:
     - Partial report continuation: rejected because mixed-validity reports are hard to trust.

## Risks / Trade-offs

- [Run matrix too small to generalize] -> Mitigation: keep dataset fixed for PoC baseline and report representativeness limits explicitly.
- [Live latency variance masks true pipeline latency delta] -> Mitigation: report median/p90 with primary and incident-filtered secondary views.
- [Telemetry field drift across versions] -> Mitigation: version run summary schema and enforce field completeness validation before aggregation.
- [Pairing mistakes between baseline and optimized runs] -> Mitigation: key pairing by `(page_id, pattern, edit_intent_hash)` and fail batch on unmatched pairs.
- [Drift checks increase execution overhead] -> Mitigation: keep live smoke set minimal and focused on high-signal parity cases.

## Migration Plan

1. Add manifest model and CLI entrypoints for batch paired run execution.
2. Extend run summary output with all required KPI and gate fields.
3. Add artifact index builder and deterministic KPI aggregation/report generation.
4. Add drift-check workflows and scenario coverage validation hooks.
5. Add fixture/integration coverage for pass/fail reporting paths and gate failures.
6. Validate end-to-end with lint/test and sample batch replay.

Rollback strategy:
- If Phase 4 reporting logic destabilizes existing execution paths, disable batch/report commands behind a feature flag and keep prior single-run behavior intact while retaining replay artifacts.

## Open Questions

- Should recommendation generation be strictly automatic (`go|iterate|stop`) or include human override metadata in the final report?
- What minimum live smoke cadence is acceptable during PoC (per-batch vs daily) without delaying iteration?
- Do we need a distinct error code for telemetry incompleteness to separate instrumentation failures from pipeline behavior failures?
