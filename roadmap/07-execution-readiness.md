# Execution Readiness (v1)

## Objective

Define the minimum operational, quality, and governance conditions required to run the v1 PoC and make a defensible `go | iterate | stop` decision.

## Scope

- Applies to v1 single-page scoped update flow plus lifecycle release-enablement checks.
- Uses KPI and experiment protocol from `04-kpi-and-experiments.md`.
- Uses risk controls from `05-risks-and-mitigations.md`.
- Uses pipeline and contract rules from `02-solution-architecture.md` and `09-ai-contract-spec.md`.
- Uses test environment and simulation policy from `10-testing-strategy-and-simulation.md`.

## Rust Implementation Blueprint (draft-first)

- Workspace shape:
  - `crates/atlassy-cli` for operator-facing commands
  - `crates/atlassy-pipeline` for state orchestration
  - `crates/atlassy-adf` for routing, patch ops, and lock fingerprints
  - `crates/atlassy-confluence` for API integration and publish/retry handling
  - `crates/atlassy-contracts` for state envelope types and validation
- Default library stack: `clap`, `tokio`, `reqwest`, `serde`, `serde_json`, `tracing`, `thiserror`.
- Test model: fixture-backed `cargo test` with deterministic replay artifacts.
- Markdown assist detail policy: keep v1 behavior per route constraints; refine implementation details after Phase 1 skeleton is stable.

## OpenSpec Control Plan (execution)

- OpenSpec is the execution controller; roadmap docs remain strategic baseline.
- Planned change IDs:
  - `phase1-core-pipeline-skeleton-rust`
  - `phase2-prose-assist-route-rust`
  - `phase3-table-cell-route-rust`
  - `phase4-poc-execution-metrics-rust`
  - `phase5-hardening-readiness-rust`
- Work rule: no implementation task starts without a corresponding OpenSpec change artifact set (`proposal`, `specs`, `design`, `tasks`).

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
- Deferred capabilities are explicitly out of scope, except approved lifecycle release-enablement defaults.

### Gate 2: Environment and Access

- Confluence test dataset is reachable and stable.
- Credentials/secrets are provisioned in approved runtime paths.
- Artifact storage path is writable for temporary execution output (`artifacts/`) and replay generation.
- Clock synchronization and timestamps are reliable for latency metrics.
- Rust toolchain (`rustc`, `cargo`) is installed and pinned for the repository runtime.
- Dedicated sandbox write access is available for controlled live research probes.
- Sandbox permissions include child-page creation under designated parent pages.

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
- Stub simulation scenarios cover all required v1 hard-error paths.
- Live smoke checks are defined for behavior drift detection.
- Lifecycle matrix tests are defined and reproducible (blank subpage create, bootstrap required fail, bootstrap success, bootstrap-on-non-empty hard fail).

### Gate 5: Metrics and Reporting

- Required per-run telemetry fields are complete.
- Per-run provenance fields are complete (`git_commit_sha`, `git_dirty`, `pipeline_version`).
- Baseline and optimized runs are paired by page and edit intent hash.
- Aggregate report computes median and p90 for all KPIs.
- Outlier handling includes primary and incident-filtered secondary views.

## Evidence Provenance Requirements

- Any KPI or readiness claim must be traceable to a specific commit SHA.
- Evidence packets must include:
  - `git_commit_sha` (full 40-character SHA)
  - `git_dirty` (working tree cleanliness marker)
  - `pipeline_version`
  - command set used to regenerate outputs
- Reports without commit provenance are non-authoritative for decision sign-off.

### Gate 6: Risk Control Activation

- All high-priority open risks have active mitigation actions.
- Alert thresholds are defined for key leading signals.
- Triage runbook exists for verify fail and conflict retry exhaustion.
- Escalation owners are assigned for blocked publish events.

### Gate 7: Lifecycle Enablement Validation

- `create-subpage` produces truly blank child pages with deterministic metadata.
- Empty-page first edit without `--bootstrap-empty-page` hard-fails deterministically.
- Empty-page first edit with `--bootstrap-empty-page` succeeds without route-policy regression.
- Bootstrap on non-empty page hard-fails deterministically.
- Lifecycle evidence bundle is committed with clean provenance for decision review.

## Pre-Run Checklist

- Confirm dataset page list and pattern matrix for the batch.
- Confirm model/runtime configuration is unchanged across paired runs.
- Confirm logging schema validation is active.
- Confirm retry policy is configured to one scoped retry.
- Confirm rollback and failure artifact retention settings.
- Confirm sandbox parent-page IDs and lifecycle test pages are prepared for create/bootstrap checks.

## Batch Runbook

1. Execute lifecycle matrix checks and record expected outcomes before paired KPI runs.
2. Start batch with run manifest (`run_id`, page, pattern, flow, intent hash).
3. Execute paired baseline/optimized runs with alternating order.
4. Validate run record completeness immediately after each run.
5. Triage hard failures by error code before continuing large batches.
6. Publish batch summary and artifact index at batch close.

## Go/No-Go Decision Criteria

Decision uses KPI and safety gates together:

- `go`: all PoC pass rules in `04-kpi-and-experiments.md` pass, lifecycle matrix checks pass, and no unresolved safety violations.
- `iterate`: safety gates hold, but one or more KPI targets miss and can be corrected without changing v1 scope.
- `stop`: safety gates fail or target recovery requires expanding scope beyond v1 defaults.

## Decision Meeting Inputs

- Latest aggregate KPI report.
- Provenance stamp for evidence build (`git_commit_sha`, `git_dirty`, `pipeline_version`).
- Risk register delta from `05-risks-and-mitigations.md`.
- Lifecycle validation evidence and matrix outcome summary.
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
