# Execution Readiness

## Objective

Define the minimum operational, quality, and governance conditions required to run PoC experiments and make defensible `go | iterate | stop` decisions for each phase.

## Scope

- Foundation (Gates 1-7): single-page scoped text-replacement flow plus lifecycle release-enablement checks.
- Structural (Gates 8-10): block operation integrity, structural composition validation, and multi-page orchestration safety.
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
  - `phase1-core-pipeline-skeleton-rust` (Foundation, complete)
  - `phase2-prose-assist-route-rust` (Foundation, complete)
  - `phase3-table-cell-route-rust` (Foundation, complete)
  - `phase4-poc-execution-metrics-rust` (Foundation, complete)
  - `phase5-hardening-readiness-rust` (Foundation, complete)
  - `phase5.5-structural-refactor` (Structural, planned)
  - `phase6-block-operation-foundation` (Structural, planned)
  - `phase7-structural-composition` (Structural, planned)
  - `phase8-multi-page-content-control` (Structural, planned)
  - `phase9-advanced-operations` (Structural, planned)
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
- Scope resolution unit tests cover `heading:` section extraction (heading with trailing content, heading at end of array, adjacent headings, nested content, multi-selector merge).
- At least one pipeline integration test uses non-empty `scope_selectors` and verifies `scope_resolution_failed: false`, `context_reduction_ratio > 0`, and downstream states succeed.
- `block:` selector has unit test coverage for `attrs.id` and `attrs.localId` matching.

### Gate 5: Metrics and Reporting

- Required per-run telemetry fields are complete, including `full_page_adf_bytes`, `scoped_adf_bytes`, `context_reduction_ratio`, `patch_ops_bytes`, and `retry_count`.
- Per-run provenance fields are complete (`git_commit_sha`, `git_dirty`, `pipeline_version`).
- Baseline and optimized runs are paired by page and edit intent hash.
- Baseline runs use empty `scope_selectors`; optimized runs use explicit heading/block selectors.
- Aggregate report computes median and p90 for all KPIs and includes per-page context-reduction distributions.
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

### Gate 7.5: Refactoring Integrity (Phase 5.5)

- All 159 existing tests pass with zero behavior change after type consolidation.
- `Operation::Replace` produces byte-identical ADF output to previous `PatchOperation { op: "replace" }`.
- `PatchCandidate`, `PatchOperation`, and `PatchOp` types are removed from the codebase.
- `AdfBlockOps` pipeline state exists as a no-op pass-through.
- Verify stage uses extracted check functions with identical behavior to previous monolithic function.
- No new features, no new operation types, no new error codes.
- `cargo test --workspace` passes with zero warnings.

### Gate 8: Block Operation Integrity (Phase 6)

- Insert operations produce schema-valid ADF; post-mutation validation passes for all insert/delete runs.
- Delete operations remove exactly the target block with no side effects on adjacent blocks.
- Reverse-order processing produces correct results for multi-operation batches (insert + delete + replace in same run).
- Existing v1 replace operations are unchanged (backward compatibility confirmed by v1 test suite).
- New error codes (`ERR_INSERT_POSITION_INVALID`, `ERR_REMOVE_ANCHOR_MISSING`, `ERR_POST_MUTATION_SCHEMA_INVALID`) fire correctly for invalid operations.
- Operation manifest cross-check in verify stage detects unmatched changes.
- Structural KPI targets met: `operation_success_rate` >95%, `schema_validity_rate` 100%, `operation_precision` 100%.

### Gate 9: Structural Composition Validation (Phase 7)

- Section insert (heading + body blocks as unit) produces valid page structure and publishes.
- Section delete (heading + all body content) removes exactly the target section.
- Table creation (new table with specified dimensions) produces valid ADF.
- List creation (new list with specified items) produces valid ADF.
- Scope boundaries are respected for all compound structural operations.
- `locked_structural` relaxation for insert/delete does not affect container wrapper preservation.
- Template blocks produce consistent, schema-valid output.

### Gate 10: Multi-Page Orchestration Safety (Phase 8)

- Multi-page rollback works correctly on partial failure: completed pages revert to pre-operation state.
- Page hierarchy awareness produces correct dependency ordering for cross-page operations.
- Content-bearing sub-page creation publishes valid ADF structure.
- Provenance tracking spans multi-page operations with per-page and batch-level metadata.
- No orphaned state: partial failures do not leave pages in inconsistent intermediate states.
- `ERR_MULTI_PAGE_PARTIAL_FAILURE` fires correctly and includes per-page failure details.

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

### Foundation (Phases 0-5)

- `go`: all Foundation PoC pass rules in `04-kpi-and-experiments.md` pass, lifecycle matrix checks pass, and no unresolved safety violations.
- `iterate`: safety gates hold, but one or more KPI targets miss and can be corrected within Foundation scope.
- `stop`: safety gates fail or target recovery requires expanding scope beyond Foundation defaults.

### Structural (Phases 6-9)

- `go`: all Structural pass rules in `04-kpi-and-experiments.md` pass, phase-specific gate (8/9/10) passes, and no unresolved safety violations.
- `iterate`: safety gates hold, but one or more Structural KPI targets miss and can be corrected within current phase scope.
- `stop`: safety gates fail, post-mutation schema validation cannot be guaranteed, or structural operations produce non-deterministic results.

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
