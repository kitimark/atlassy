# Risks and Mitigations (v1)

## Objective

Define the primary delivery and quality risks for v1, with concrete controls, detection signals, and response actions that preserve safety defaults.

## Scope and Assumptions

- Scope is limited to v1 pipeline behavior and PoC execution.
- Route policy and safety gates follow `06-decisions-and-defaults.md`.
- KPI pass/fail logic follows `04-kpi-and-experiments.md`.
- Deferred capabilities remain out of scope unless formally promoted.

## Risk Rating Model

- Likelihood: `L` (low), `M` (medium), `H` (high).
- Impact: `L` (low), `M` (medium), `H` (high).
- Priority: qualitative combination of likelihood and impact.
- Risk status: `open`, `watch`, `mitigated`, `accepted`, `closed`.

## v1 Risk Register

### R-001 Out-of-scope mutation

- Description: candidate patch mutates paths outside `allowed_scope_paths`.
- Likelihood: M.
- Impact: H.
- Priority: high.
- Preventive controls:
  - Path-targeted patch operations only.
  - Hard verifier gate for out-of-scope mutation.
  - Deterministic `changed_paths` uniqueness and sorting checks.
- Detection signals:
  - `ERR_OUT_OF_SCOPE_MUTATION`.
  - Unexpected growth in changed path count per run.
- Response:
  - Block publish, persist artifacts, and mark run failed.
  - Add fixture reproducing the mutation path.

### R-002 Locked structural node mutation

- Description: media/macro/layout or other locked nodes are changed unintentionally.
- Likelihood: M.
- Impact: H.
- Priority: high.
- Preventive controls:
  - Strict route enforcement (`locked_structural` non-editable).
  - Locked-node fingerprint verification before publish.
  - No markdown conversion for non-prose routes.
- Detection signals:
  - `ERR_LOCKED_NODE_MUTATION`.
  - Drops in `formatting_fidelity_pass_rate`.
- Response:
  - Block publish and open defect with offending node path.
  - Extend route-classifier tests for the missed node type.

### R-003 Table shape drift under v1 constraints

- Description: table edit path introduces row/column topology or attribute changes.
- Likelihood: M.
- Impact: H.
- Priority: high.
- Preventive controls:
  - Allow only `cell_text_update` in `adf_table_edit`.
  - Explicit rejection of row/column add/remove and merge/split ops.
  - Table node diff checks in verifier.
- Detection signals:
  - `ERR_TABLE_SHAPE_CHANGE`.
  - Publish failures clustered on Pattern B runs.
- Response:
  - Reject publish and keep candidate patch for analysis.
  - Add fixture coverage for complex table variants.

### R-004 Full-page retrieval fallback overuse

- Description: frequent scope misses force full-page retrieval and erase token savings.
- Likelihood: M.
- Impact: M.
- Priority: medium.
- Preventive controls:
  - Strengthen heading/block scope resolution.
  - Add selector validation before fetch execution.
  - Log fallback reason codes for every full-page fetch.
- Detection signals:
  - Rising `full_page_retrieval_rate`.
  - Increased `ERR_SCOPE_MISS` frequency.
- Response:
  - Triage top fallback causes by selector type.
  - Fix resolver logic before broadening datasets.

### R-005 Conflict retry token waste

- Description: version conflicts consume excessive tokens or cause hidden retry loops.
- Likelihood: M.
- Impact: M.
- Priority: medium.
- Preventive controls:
  - One scoped rebase retry maximum.
  - Fail fast after retry exhaustion.
  - Track retry-only token spend per run.
- Detection signals:
  - `ERR_CONFLICT_RETRY_EXHAUSTED`.
  - Rising `retry_conflict_token_waste`.
- Response:
  - Stop at first exceeded retry condition.
  - Queue reviewer artifact with conflict diagnostics.

### R-006 Schema-invalid candidate payloads

- Description: generated candidate ADF fails schema validation at verify time.
- Likelihood: M.
- Impact: H.
- Priority: high.
- Preventive controls:
  - Schema validation before publish call.
  - Route-aware candidate merge checks.
  - Strict state contract validation per step.
- Detection signals:
  - `ERR_SCHEMA_INVALID`.
  - Repeated verify failures on a node family.
- Response:
  - Block publish and capture invalid fragment path.
  - Add regression fixture for failing structure.

### R-007 Metrics instrumentation gaps

- Description: missing or inconsistent telemetry prevents trustworthy KPI conclusions.
- Likelihood: M.
- Impact: M.
- Priority: medium.
- Preventive controls:
  - Required run fields enforced by schema.
  - Reject incomplete run summaries from aggregate reports.
  - Persist replay artifacts per state.
- Detection signals:
  - Missing `run_id` or state token fields.
  - Divergence between summary totals and per-state totals.
- Response:
  - Mark run as non-evaluable and rerun paired case.
  - Fix instrumentation before pass/fail decisions.

### R-008 External service variance masking regressions

- Description: Confluence latency/outage noise obscures real flow performance.
- Likelihood: M.
- Impact: M.
- Priority: medium.
- Preventive controls:
  - Paired baseline/optimized sequencing.
  - Alternate run order to reduce order effects.
  - Track service incident markers per run.
- Detection signals:
  - Large latency spikes with known service incidents.
  - Inconsistent KPI deltas across identical intents.
- Response:
  - Report primary view with all runs.
  - Add secondary incident-filtered view, never hiding verifier failures.

## Deferred-Scope Risk Notes

- Advanced table operations are deferred to `ideas/2026-03-advanced-table-editing-modes.md`.
- Structural block editing is deferred to `ideas/2026-03-structural-block-editing-support.md`.
- Multi-page orchestration is deferred to `ideas/2026-03-multi-page-orchestration-and-autonomous-conflict-resolution.md`.

Any pull-in of these scopes requires a decision-log update and new verifier rules before implementation.

## Operational Mitigation Playbook

- `verify` fail: block publish, persist artifacts, label root cause by error code.
- `publish` conflict after retry: fail fast, return reviewer artifact, no additional retries.
- Repeated route violations: freeze new feature work for targeted classifier/verifier hardening.
- KPI regression with stable verification: run incident-filtered secondary analysis, then decide `iterate` or `stop`.

## Governance and Review Cadence

- Review risk register at the end of each phase in `03-phased-roadmap.md`.
- Re-score likelihood/impact after each PoC batch report.
- Promote `watch` risks to `open` when a leading signal exceeds threshold for two consecutive batches.
- Record material changes in `06-decisions-and-defaults.md`.

## Exit Criteria for v1 Risk Readiness

- No unresolved high-priority risk without an active mitigation plan.
- No evidence of locked-node mutation in PoC runs.
- Conflict behavior remains bounded to one scoped retry.
- KPI evidence is complete enough to support `go`, `iterate`, or `stop` recommendation.
