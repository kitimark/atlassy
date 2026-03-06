# Live Runtime Execution Plan

## Objective

Move Atlassy from stub-validated execution to real Confluence pilot readiness while preserving safety, replayability, and deterministic decision evidence.

## Current Baseline

- Pipeline states and batch/readiness flows are implemented and runnable in stub mode.
- Current readiness recommendation is `iterate` due to KPI misses.
- `artifacts/` is temporary runtime output and is not versioned.
- Decision-grade evidence must include:
  - `git_commit_sha` (40-char SHA)
  - `git_dirty` (true/false)
  - `pipeline_version`

## Scope

### In Scope

1. Provenance in run/batch/readiness outputs.
2. Patch-stage correctness (`patch_ops` reflected in candidate ADF).
3. Live Confluence runtime implementation and runtime backend selection (`stub|live`).
4. Telemetry quality improvements for KPI validity.
5. End-to-end gated rerun of validation sequence.

### Out of Scope

- New editing capabilities beyond current v1 scope.
- Multi-page orchestration.
- Long-term analytics platform work.

## Work Packages (Strict Order)

### WP1: Provenance in Decision-Grade Outputs

**Goal**

- Ensure all reported results are traceable to an exact implementation revision.

**Required Outcomes**

- Per-run summary includes `git_commit_sha`, `git_dirty`, `pipeline_version`.
- Batch and readiness outputs include the same provenance stamp.

**Done Criteria**

- Sample run summary and decision packet both contain provenance fields.
- No KPI/readiness claim is emitted without provenance.

### WP2: Patch Application Correctness

**Goal**

- Ensure generated `patch_ops` are applied into the candidate payload before verify/publish.

**Required Outcomes**

- Patch stage mutates `candidate_page_adf` from `patch_ops`.
- Behavior is correct for prose update and table-cell update paths.

**Done Criteria**

- Patch artifacts show mutated candidate payload matching patch operations.
- Regression tests confirm unchanged paths remain unchanged.

### WP3: Live Confluence Runtime and Backend Selection

**Goal**

- Enable real Confluence fetch/publish while preserving stub compatibility.

**Required Outcomes**

- Live fetch/publish paths implemented with deterministic error mapping.
- Runtime backend selector added (`stub|live`) with environment-driven config.
- Conflict behavior aligns with one scoped retry policy.

**Done Criteria**

- Stub mode still passes existing checks.
- Live sandbox fetch/publish succeeds.
- Conflict scenario follows scoped retry limits.

### WP4: Telemetry Quality for KPI Validity

**Goal**

- Replace placeholder telemetry with meaningful operational metrics.

**Required Outcomes**

- Accurate timing and retry metrics in run outputs.
- Token accounting and scope retrieval metrics are consistently populated.
- Batch KPI aggregation remains deterministic.

**Done Criteria**

- KPI report fields are not static/placeholder unless expected by scenario.
- Telemetry completeness checks continue to pass.

### WP5: Gated Validation Rerun

**Goal**

- Revalidate behavior with the existing checkpoint sequence.

**Execution Sequence**

1. Smoke run (no-op)
2. Scoped prose update run
3. Scoped table-cell update run
4. Negative safety run (expected failure)
5. Batch run
6. Readiness run with replay verification

**Done Criteria**

- Safety-negative run blocks publish with deterministic error code.
- Replay verification passes.
- Decision output is evidence-backed and provenance-stamped.

## Stop Conditions (Immediate Triage)

Stop execution and triage if any occur:

- Locked-node mutation detection.
- Retry policy breach (> one scoped retry).
- Missing provenance in decision-grade outputs.
- Replay mismatch for rebuilt decision outputs.
- Unmapped hard errors in live publish path.

## Exit Criteria (Pilot-Ready)

This plan is complete when all conditions hold:

1. Provenance is present in run, batch, and readiness outputs.
2. Patch stage candidate payload reflects patch operations.
3. Live sandbox fetch/publish works in `live` mode.
4. Gated validation sequence completes with deterministic evidence.
5. Final recommendation is documented with explicit blocking reasons (if non-go).
