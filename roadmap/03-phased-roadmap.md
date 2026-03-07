# Phased Roadmap (v1)

## Objective

Deliver a token-efficient, minimal-change Confluence update pipeline that preserves ADF fidelity while meeting v1 safety and reliability constraints.

## Guiding Constraints

- ADF remains canonical for `fetch`, `patch`, `verify`, and `publish`.
- Markdown is assist-only for `editable_prose`.
- Tables in v1 are ADF-native and limited to cell text updates.
- Locked structural nodes must remain unchanged.
- Publish conflicts allow one scoped rebase retry, then fail fast.

## Phase Overview

- Phase 0: Design baseline (complete)
- Phase 1: Core pipeline skeleton
- Phase 2: Prose assist route
- Phase 3: Table cell edit route
- Phase 4: PoC execution and metrics validation
- Phase 5: Hardening and v1 readiness

## Implementation Checkpoint (2026-03-07)

- Phase containers for phases 1-5 are implemented and archived under `openspec/changes/archive/`.
- Stub and fixture-backed execution is operational (`run`, `run-batch`, `run-readiness`).
- Live Confluence runtime (`LiveConfluenceClient`) is operational and validated in sandbox with committed evidence.
- Lifecycle features (`create-subpage`, `--bootstrap-empty-page`) are implemented, tested, and validated against live Confluence.
- Lifecycle matrix evidence is committed under `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/`.
- Current recommendation remains `iterate`, driven by KPI misses in token reduction and full-page retrieval reduction (stub-backed evaluation; live KPI revalidation is pending).

## Phase 0: Design Baseline

### Scope

- Finalize architecture, defaults, PoC scope, and AI-facing contracts.
- Align route matrix and verifier gates across planning docs.
- Define success metrics and exit criteria for PoC.

### Deliverables

- `01-problem-points.md`
- `02-solution-architecture.md`
- `03-phased-roadmap.md`
- `04-kpi-and-experiments.md`
- `05-risks-and-mitigations.md`
- `06-decisions-and-defaults.md`
- `07-execution-readiness.md`
- `08-poc-scope.md`
- `09-ai-contract-spec.md`

### Exit Criteria

- Decision log reflects all v1 defaults.
- AI state contracts are versioned and internally consistent.
- PoC patterns and success targets are explicitly documented.

## Phase 1: Core Pipeline Skeleton

### Scope

- Implement orchestration for all v1 states (`fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> merge_candidates -> patch -> verify -> publish`).
- Implement Phase 1 runtime using Rust workspace foundations and typed state envelopes.
- Establish diagnostics and replay artifact persistence.
- Support scoped ADF retrieval and node-path indexing.

### Acceptance Criteria

- End-to-end no-op and simple scoped update flows complete.
- Whole-body rewrite attempts are rejected.
- Hard errors halt pipeline with deterministic error codes.
- Replay artifacts persist per state.

## Phase 2: Prose Assist Route

### Scope

- Implement `extract_prose` and `md_assist_edit` for `editable_prose` only.
- Preserve stable markdown block to ADF path mapping.
- Enforce prose-boundary and top-level type constraints.

### Acceptance Criteria

- Prose edits apply only to mapped prose paths.
- No table or locked nodes are converted to markdown.
- Out-of-scope mutation is detected and blocked by `verify`.
- Prose formatting fidelity is non-regressive on fixtures.

## Phase 3: Table Cell Edit Route

### Scope

- Implement `adf_table_edit` for table cell text updates only.
- Merge table candidates with path uniqueness and conflict checks.
- Reject table topology and attribute changes in v1.

### Acceptance Criteria

- Allowed op remains `cell_text_update` only.
- Forbidden table ops return `ERR_TABLE_SHAPE_CHANGE`.
- Cross-route conflicts fail fast at merge.
- Table edits publish without structural drift.

## Phase 4: PoC Execution and Metrics Validation

### Scope

- Execute dataset and Pattern A/B/C scenarios from `08-poc-scope.md`.
- Run live Confluence behavior probes in sandbox and align stub simulation scenarios.
- Run paired baseline vs optimized experiments.
- Produce batch and aggregate KPI reports.

### Targets

- `tokens_per_successful_update`: 40-60% reduction
- `full_page_retrieval_rate`: 60-80% reduction
- `formatting_fidelity_pass_rate`: non-regressive
- `publish_latency`: non-regressive
- `retry_conflict_token_waste`: bounded by one retry policy

### Exit Criteria

- In-scope patterns pass verifier checks.
- No locked-node mutation appears in logs.
- Conflict behavior remains bounded to one scoped retry.
- Outcomes are recorded in decision updates.

## Phase 5: Hardening and v1 Readiness

### Scope

- Stabilize error handling, observability, and operator guidance.
- Address PoC gaps with non-breaking v1 refinements.
- Implement lifecycle release-enablement features from `12-page-lifecycle-expansion-plan.md`.
- Complete readiness checklist and decision sign-off.

### Acceptance Criteria

- Failure modes map to clear operator actions.
- Metrics collection is reproducible and complete.
- Lifecycle matrix passes in sandbox (`create-subpage` blank creation, empty-page bootstrap required fail, empty-page bootstrap success, bootstrap-on-non-empty hard fail).
- Readiness checklist is signed.
- Final recommendation is documented (`go | iterate | stop`).

## Dependencies and Planning Tracks

- Problem framing: `01-problem-points.md`
- KPI protocol: `04-kpi-and-experiments.md`
- Risk controls: `05-risks-and-mitigations.md`
- Execution checklist: `07-execution-readiness.md`
- Testing strategy and simulation: `10-testing-strategy-and-simulation.md`
- Lifecycle release-enablement track: `12-page-lifecycle-expansion-plan.md`

## OpenSpec Change Map

- `phase1-core-pipeline-skeleton-rust`
- `phase2-prose-assist-route-rust`
- `phase3-table-cell-route-rust`
- `phase4-poc-execution-metrics-rust`
- `phase5-hardening-readiness-rust`

These change IDs are planned execution containers under OpenSpec and should be used to track proposal/design/tasks and implementation progress.

## Explicitly Deferred Beyond v1

- Table row/column add/remove
- Table merge/split and table attribute changes
- Structural transformations for macros/media/layouts
- Multi-page orchestration and autonomous conflict resolution

See:

- `ideas/2026-03-advanced-table-editing-modes.md`
- `ideas/2026-03-structural-block-editing-support.md`
- `ideas/2026-03-multi-page-orchestration-and-autonomous-conflict-resolution.md`
