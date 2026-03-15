# Risks and Mitigations

## Objective

Define the primary delivery and quality risks for Foundation and Structural phases, with concrete controls, detection signals, and response actions that preserve safety defaults.

## Scope and Assumptions

- Foundation risks (R-001 through R-010): pipeline behavior and PoC execution.
- Structural risks (R-011 through R-014): structural operations, index shift handling, multi-page orchestration.
- Route policy and safety gates follow `06-decisions-and-defaults.md`.
- KPI pass/fail logic follows `04-kpi-and-experiments.md`.
- Deferred capabilities remain out of scope unless formally promoted.
- Lifecycle release-enablement capabilities in `12-page-lifecycle-expansion-plan.md` are in scope for v1 release gating.

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
  - Drops in `structural_preservation`.
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
  - `context_reduction_ratio` falling below target band (<70% median in optimized runs).
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
  - Rising `conflict_rate`.
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
  - Missing `run_id`, `full_page_adf_bytes`, `scoped_adf_bytes`, or `context_reduction_ratio`.
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

### R-009 Sub-page creation misplacement or duplication

- Description: lifecycle page creation produces wrong parent placement or duplicate-title ambiguity.
- Likelihood: M.
- Impact: M.
- Priority: medium.
- Preventive controls:
  - Require explicit parent page ID for `create-subpage`.
  - Enforce deterministic duplicate-title handling policy.
  - Emit provenance fields for parent and created page IDs.
- Detection signals:
  - Repeated create failures for duplicate-title or permission classes.
  - Unexpected page placement discovered in sandbox audits.
- Response:
  - Stop automated lifecycle run, correct parent/title policy, and rerun.
  - Add regression coverage for parent/duplicate failure paths.

### R-010 Empty-page bootstrap precondition drift

- Description: empty/non-empty detection or bootstrap precondition handling becomes non-deterministic.
- Likelihood: M.
- Impact: H.
- Priority: high.
- Preventive controls:
  - Enforce explicit bootstrap behavior matrix with hard-fail preconditions.
  - Add regression tests for all four lifecycle matrix outcomes.
  - Persist bootstrap telemetry markers for each run.
- Detection signals:
  - Missing expected bootstrap failures on non-empty pages.
  - Unexpected route-policy or schema violations after bootstrap.
- Response:
  - Pause lifecycle release gating and triage bootstrap state classification.
  - Restore deterministic hard-fail behavior before readiness continuation.

## Structural Risk Register (Phases 6-9)

### R-011 ADF schema corruption from insert/delete

- Description: insert or delete operations produce ADF that violates the Confluence schema, causing publish rejection or page corruption.
- Likelihood: M.
- Impact: H.
- Priority: high.
- Preventive controls:
  - Post-mutation ADF schema validation required before publish (D-017).
  - Reverse-order processing prevents cascading index shifts (D-020).
  - Operation manifest cross-check in verify stage.
- Detection signals:
  - `ERR_POST_MUTATION_SCHEMA_INVALID`.
  - Publish rejections with Confluence-side schema errors.
- Response:
  - Block publish, persist pre- and post-mutation ADF for analysis.
  - Add fixture reproducing the invalid mutation.
  - Verify schema validation coverage against ADF JSON Schema reference.

### R-012 Cascading index shift errors

- Description: insert or delete at one position invalidates paths for subsequent operations in the same batch.
- Likelihood: M.
- Impact: H.
- Priority: high.
- Preventive controls:
  - Reverse-document-order processing (D-020).
  - Operations within same parent array sorted by descending index.
  - Integration tests with multi-operation batches targeting same parent.
- Detection signals:
  - Mismatched paths between operation manifest and actual changes.
  - `ERR_INSERT_POSITION_INVALID` or `ERR_REMOVE_ANCHOR_MISSING` on valid-looking operations.
  - `operation_precision` below 100%.
- Response:
  - Halt batch, dump operation ordering and path state.
  - Add regression test for the failing operation combination.
  - Evaluate stable-ID addressing if reverse-order proves insufficient.

### R-013 Multi-page partial failure

- Description: coordinated multi-page operation fails partway through, leaving some pages updated and others not.
- Likelihood: M.
- Impact: H.
- Priority: high.
- Preventive controls:
  - Rollback checkpoints per page (Phase 8).
  - Page-level isolation: each page operation is independently verifiable before batch commits.
  - Atomic commit semantics: all pages succeed or all revert.
- Detection signals:
  - `ERR_MULTI_PAGE_PARTIAL_FAILURE`.
  - Inconsistent page versions across a coordinated edit set.
- Response:
  - Trigger rollback for all completed pages.
  - Persist per-page operation state for diagnosis.
  - Require explicit operator approval before retrying multi-page operations.

### R-014 Scope anchor deletion paradox

- Description: deleting a heading that serves as a scope anchor makes subsequent scope resolution fail for the same selector.
- Likelihood: L.
- Impact: M.
- Priority: medium.
- Preventive controls:
  - Scope anchors (headings used in `scope_selectors`) blocked from deletion by default (D-018).
  - Explicit re-scoping required when deleting a scope anchor.
  - Pre-operation scope anchor detection before processing delete ops.
- Detection signals:
  - `ERR_REMOVE_ANCHOR_MISSING` or scope resolution failure after deletion.
  - Scope fallback to full-page fetch triggered by anchor removal.
- Response:
  - Block the delete operation with clear error message.
  - Operator must provide updated scope selectors if anchor deletion is intended.

### R-015 Refactoring regression during type consolidation

- Description: Phase 5.5 type consolidation (replacing 3 types with 1 `Operation` enum) introduces subtle behavioral changes despite the intent of zero behavior change.
- Likelihood: L.
- Impact: H.
- Priority: medium.
- Preventive controls:
  - All 159 existing tests must pass before and after refactoring (Gate 7.5).
  - Byte-identical ADF output verification for `Operation::Replace` vs previous `PatchOperation`.
  - Incremental refactoring: one type replacement at a time, tests run after each step.
  - No new features mixed with refactoring work (D-021).
- Detection signals:
  - Any test failure during Phase 5.5 indicates incorrect refactoring.
  - ADF output diff between pre- and post-refactoring runs.
- Response:
  - Revert the specific refactoring step that caused the failure.
  - Add a regression test capturing the failure before re-attempting.

## Promoted Scope Risk Notes

The following previously deferred capabilities have been promoted into the Structural phases roadmap:

- Advanced table operations: promoted to Phase 9 (from `ideas/2026-03-advanced-table-editing-modes.md`). Risks covered by R-003 (existing) and R-011 (new).
- Structural block editing: promoted to Phase 7 and Phase 9 (from `ideas/2026-03-structural-block-editing-support.md`). Risks covered by R-011 and R-012.
- Multi-page orchestration: promoted to Phase 8 (from `ideas/2026-03-multi-page-orchestration-and-autonomous-conflict-resolution.md`). Risks covered by R-013.

Lifecycle release-enablement reference (in v1 scope):

- `roadmap/12-page-lifecycle-expansion-plan.md`

## Remaining Deferred-Scope Risk Notes

- Block type conversion (paragraph to heading, etc.) remains deferred.
- Inline node editing (mentions, status, emoji) remains deferred.
- Multi-space orchestration remains deferred.
- Stable-ID addressing is a potential future upgrade if reverse-order processing proves insufficient (D-020).

Any pull-in of deferred scopes requires a decision-log update and new verifier rules before implementation.

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
