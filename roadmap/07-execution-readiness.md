# Execution Readiness (v1)

## Objective

Define the minimum operational, quality, and governance conditions required to run the v1 PoC and make a defensible `go | iterate | stop` decision.

## Scope

- Applies to v1 single-page scoped update flow only.
- Uses KPI and experiment protocol from `04-kpi-and-experiments.md`.
- Uses risk controls from `05-risks-and-mitigations.md`.
- Uses pipeline and contract rules from `02-solution-architecture.md` and `09-ai-contract-spec.md`.

## Roles and Ownership

- Product owner: approves PoC objective, success targets, and final recommendation.
- Engineering owner: accountable for pipeline implementation and defect triage.
- QA owner: accountable for fixture coverage, verifier outcomes, and reproducibility checks.
- Data/metrics owner: accountable for run telemetry quality and KPI report accuracy.
- Release reviewer: validates checklist completion and signs off on decision outcome.

## Readiness Gates

### Gate 1: Design and Contract Freeze

- v1 route matrix is frozen (`editable_prose`, `table_adf`, `locked_structural`).
- State contracts and error taxonomy are versioned and shared.
- Deferred capabilities are explicitly out of scope.

### Gate 2: Environment and Access

- Confluence test dataset is reachable and stable.
- Credentials/secrets are provisioned in approved runtime paths.
- Artifact storage path is writable and retained for replay.
- Clock synchronization and timestamps are reliable for latency metrics.

### Gate 3: Pipeline Integrity

- All required states run in order with contract-valid payloads.
- Path-targeted patching is enforced; whole-body rewrite blocked.
- Verifier hard-fails on schema, route, scope, and locked-node violations.
- Publish conflict policy enforces one scoped retry maximum.

### Gate 4: Test and Fixture Coverage

- Pattern A/B/C fixtures are available and reproducible.
- Positive tests pass for prose-only and mixed prose/table edits.
- Negative tests pass for forbidden table operations and out-of-scope mutations.
- Replay artifacts can reproduce at least one failure in each major error class.

### Gate 5: Metrics and Reporting

- Required per-run telemetry fields are complete.
- Baseline and optimized runs are paired by page and edit intent hash.
- Aggregate report computes median and p90 for all KPIs.
- Outlier handling includes primary and incident-filtered secondary views.

### Gate 6: Risk Control Activation

- All high-priority open risks have active mitigation actions.
- Alert thresholds are defined for key leading signals.
- Triage runbook exists for verify fail and conflict retry exhaustion.
- Escalation owners are assigned for blocked publish events.

## Pre-Run Checklist

- Confirm dataset page list and pattern matrix for the batch.
- Confirm model/runtime configuration is unchanged across paired runs.
- Confirm logging schema validation is active.
- Confirm retry policy is configured to one scoped retry.
- Confirm rollback and failure artifact retention settings.

## Batch Runbook

1. Start batch with run manifest (`run_id`, page, pattern, flow, intent hash).
2. Execute paired baseline/optimized runs with alternating order.
3. Validate run record completeness immediately after each run.
4. Triage hard failures by error code before continuing large batches.
5. Publish batch summary and artifact index at batch close.

## Go/No-Go Decision Criteria

Decision uses KPI and safety gates together:

- `go`: all PoC pass rules in `04-kpi-and-experiments.md` pass, with no unresolved safety violations.
- `iterate`: safety gates hold, but one or more KPI targets miss and can be corrected without changing v1 scope.
- `stop`: safety gates fail or target recovery requires expanding scope beyond v1 defaults.

## Decision Meeting Inputs

- Latest aggregate KPI report.
- Risk register delta from `05-risks-and-mitigations.md`.
- Top failure classes with root-cause summaries.
- Recommendation memo with explicit `go | iterate | stop`.

## Exit Artifacts

- Signed readiness checklist.
- Final PoC report and artifact index.
- Decision log update in `06-decisions-and-defaults.md`.
- Next-phase plan update in `03-phased-roadmap.md`.

## Contingency Triggers

- Immediate pause if locked-node mutation is detected.
- Immediate pause if retry behavior exceeds one scoped retry.
- Immediate pause if telemetry completeness drops below reportable threshold.
- Resume only after corrective action is documented and verified.
