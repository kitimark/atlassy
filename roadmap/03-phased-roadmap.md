# Phased Roadmap

## Objective

Deliver a Confluence content control pipeline that enables insert, edit, and delete of ADF blocks across pages and sub-pages, while preserving ADF fidelity and structural safety.

The Foundation phases (0-5) established a token-efficient, minimal-change text-replacement pipeline. The Structural phases (6-9) extend the system to full structural operations: block insertion, block deletion, structural composition, and multi-page orchestration.

## Guiding Constraints

### Foundation Constraints (Phases 0-5)

- ADF remains canonical for `fetch`, `patch`, `verify`, and `publish`.
- Markdown is assist-only for `editable_prose`.
- Tables in v1 are ADF-native and limited to cell text updates.
- Locked structural nodes must remain unchanged.
- Publish conflicts allow one scoped rebase retry, then fail fast.

### Structural Constraints (Phases 6-9)

- Insert and delete operations are processed in reverse document order to maintain path stability (D-020).
- Post-mutation ADF schema validation is required before publish for any insert or delete operation.
- Structural operations respect scope boundaries; no operation may affect blocks outside `allowed_scope_paths`.
- Multi-page operations require rollback checkpoints; partial failure must not leave inconsistent state.
- Existing Foundation text-replacement behavior remains backward compatible throughout Structural phases.

## Phase Overview

- Phase 0: Design baseline (complete)
- Phase 1: Core pipeline skeleton (complete)
- Phase 2: Prose assist route (complete)
- Phase 3: Table cell edit route (complete)
- Phase 4: PoC execution and metrics validation (complete)
- Phase 5: Hardening and v1 readiness (complete)
- Phase 5.5: Structural refactor (preparatory)
- Phase 6: Block operation foundation
- Phase 7: Structural composition
- Phase 8: Multi-page content control
- Phase 9: Advanced operations

## Implementation Checkpoint (2026-03-15)

### Foundation Status (Phases 0-5): Complete

- Phase containers for phases 1-5 are implemented and archived under `openspec/changes/archive/`.
- Stub and fixture-backed execution is operational (`run`, `run-batch`, `run-readiness`).
- Live Confluence runtime (`LiveConfluenceClient`) is operational and validated in sandbox with committed evidence.
- Lifecycle features (`create-subpage`, `--bootstrap-empty-page`) are implemented, tested, and validated against live Confluence.
- Lifecycle matrix evidence is committed under `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/`.
- All 7 readiness gates pass. 159 tests pass across 5 crates.
- Foundation KPI status: `iterate` (context reduction at 64.18% vs 70% target; publish latency p90 regression). Root cause: Pattern B selector strategy on mixed-content pages.
- Foundation KPI framework is superseded by Structural KPI framework (D-019). The Pattern B selector issue becomes addressable through Phase 6 structural operations rather than narrower scoped fetching.

### Structural Status (Phases 5.5-9): Planning

- Roadmap redesigned to target insert/edit/delete of ADF blocks across pages and sub-pages.
- Decisions D-017 through D-021 define the architectural approach (type consolidation, block scope, revised KPIs, reverse-order processing, preparatory refactoring).
- Phase 5.5 (Structural Refactor) is the next implementation target: type consolidation and pipeline preparation with zero behavior change.
- Phase 6 (Block Operation Foundation) follows immediately after Phase 5.5 on the refactored foundation.

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

### Blocking Prerequisites

Before paired KPI experiments can produce valid results, the following must be resolved:

- **Section extraction fix**: `resolve_scope()` must return the heading section (heading + subsequent sibling content until the next heading), not just the heading node. Without this, all optimized runs fail at `patch` with `ERR_SCHEMA_INVALID`. Location: `crates/atlassy-adf/src/lib.rs`, `find_heading_paths()` and `resolve_scope()`.
- **Section extraction unit tests**: heading with trailing content, heading at end of array, adjacent headings (empty section), nested content under heading, multi-selector merge.
- **Scoped pipeline integration test**: at least one integration test with non-empty `scope_selectors` verifying `scope_resolution_failed: false`, `context_reduction_ratio > 0`, and downstream states (`patch`, `verify`, `publish`) succeed.
- **`seed-page` CLI command**: publish arbitrary ADF JSON to an existing page, bypassing the pipeline safety envelope. Required for reproducible experiment page setup without manual Confluence UI editing or `curl`. Design: explicit opt-in command, page must already exist, full-body ADF replacement, validate ADF syntax before publish, require `--runtime-backend live`.

### Scope

- Execute dataset and Pattern A/B/C scenarios from `08-poc-scope.md`.
- Run live Confluence behavior probes in sandbox and align stub simulation scenarios.
- Run paired baseline vs optimized experiments.
- Produce batch and aggregate KPI reports.

### Targets

- `context_reduction_ratio`: 70-90% on optimized in-scope runs
- `scoped_section_tokens`: report median and p90 by pattern (diagnostic target)
- `edit_success_rate`: >95% for in-scope runs
- `structural_preservation`: 100% non-target structure preserved
- `conflict_rate`: <10%, bounded by one scoped retry policy
- `publish_latency`: median <3000 ms for scoped optimized runs; p90 non-regressive vs baseline

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
- Resolve heading selector matching policy (D-015): change to exact match by default.
- Fix `rule` node route classification drift: align spec to code (`rule` stays `locked_structural` in v1).
- Add `block:` selector test coverage (unit tests for `find_block_paths()` matching on `attrs.id` and `attrs.localId`).
- Complete readiness checklist and decision sign-off.

### Acceptance Criteria

- Failure modes map to clear operator actions.
- Metrics collection is reproducible and complete.
- Lifecycle matrix passes in sandbox (`create-subpage` blank creation, empty-page bootstrap required fail, empty-page bootstrap success, bootstrap-on-non-empty hard fail).
- Readiness checklist is signed.
- Final recommendation is documented (`go | iterate | stop`).

## Phase 5.5: Structural Refactor

### Scope

Preparatory refactoring to make Phase 6 feature work clean and sustainable. Follows refactoring.guru principles: cure existing code smells before adding new behavior. Zero behavior change — all 159 existing tests must pass identically (D-021).

- **Replace Type Code with Class** (Primitive Obsession cure): consolidate `PatchCandidate`, `PatchOperation`, and `PatchOp` into a single `Operation` enum in `atlassy-contracts` with one variant: `Operation::Replace { path, value }` (D-017 revised).
- **Inline Class** (Shotgun Surgery cure): drop `PatchCandidate` (from `atlassy-adf`) and `PatchOp` (from `atlassy-contracts`). The `Operation` enum replaces all three types as the single operation representation flowing through the pipeline.
- **Extract Method** (Divergent Change cure): split `verify.rs` into focused check functions — `check_operation_legality()` for scope/route/table guards and a stub for `check_structural_validity()` (activated in Phase 6).
- **Add Parameter**: add `block_ops: Vec<BlockOp>` field to `RunRequest` (always empty in Phase 5.5 — no processing yet).
- **New pipeline state stub**: add `AdfBlockOps` to `PipelineState` enum, positioned between `AdfTableEdit` and `MergeCandidates`. Implemented as a no-op pass-through in Phase 5.5.
- **Ordering module stub**: add `atlassy-adf/src/ordering.rs` with `sort_operations()` that returns operations unchanged (identity sort). Phase 6 adds the reverse-document-order algorithm.
- **Contract alignment**: `MergeCandidatesOutput` changes from `changed_paths: Vec<String>` to `operations: Vec<Operation>`.

### Key Decisions

- D-017 (revised): Operation type consolidation strategy
- D-021: Preparatory refactoring strategy

### Acceptance Criteria

- All 159 existing tests pass with zero behavior change.
- `Operation::Replace` produces byte-identical ADF output to previous `PatchOperation { op: "replace" }`.
- `PatchCandidate`, `PatchOperation`, and `PatchOp` types are removed from the codebase.
- `RunRequest` accepts `block_ops` field (defaults to empty vec).
- `AdfBlockOps` pipeline state exists and passes through as no-op.
- Verify stage uses extracted check functions with identical behavior.
- No new features, no new operation types, no new error codes.

## Phase 6: Block Operation Foundation

### Scope

Add insert and delete capabilities to the `Operation` enum established in Phase 5.5. All feature work builds on the refactored type system.

- Add `Operation::Insert { parent_path, index, block }` and `Operation::Remove { target_path }` variants to the existing `Operation` enum (D-017).
- Implement `apply_insert()` and `apply_remove()` functions in `atlassy-adf` patch module, alongside existing `apply_replace()`.
- Implement reverse-document-order sorting in `atlassy-adf/src/ordering.rs`: partition replaces vs structural ops, group by parent path, descending index, remove-before-insert at same index (D-020).
- Implement `AdfBlockOps` pipeline state: processes `block_ops` from `RunRequest`, validates scope/type/conflicts, produces `Operation` instances that merge with prose and table operations.
- Phase 6 scope limited to `editable_prose` types: paragraph, heading, bulletList, orderedList, listItem, blockquote, codeBlock (D-018).
- Implement `check_structural_validity()` in `atlassy-adf`: post-mutation validation that `doc.content` is non-empty, all blocks have valid types, parent-child relationships are correct, headings have `attrs.level`.
- Activate op-aware verification in `check_operation_legality()`: allow intentional structural changes from declared operations, block unintended mutations.
- New error codes: `ERR_INSERT_POSITION_INVALID`, `ERR_REMOVE_ANCHOR_MISSING`, `ERR_POST_MUTATION_SCHEMA_INVALID`.

### Key Decisions

- D-017: Operation type consolidation strategy (established in Phase 5.5)
- D-018: Block insert/delete scope (Phase 6)
- D-019: Revised KPI framework for structural operations
- D-020: Reverse-order patch processing

### Acceptance Criteria

- Insert a paragraph after an existing heading within scope: succeeds and publishes.
- Delete a paragraph within scope: succeeds and publishes.
- Insert at an invalid position (out of bounds, inside locked node): fails with `ERR_INSERT_POSITION_INVALID`.
- Delete a scope anchor heading: blocked with `ERR_REMOVE_ANCHOR_MISSING` or requires explicit re-scoping.
- Post-mutation ADF passes structural validation for all insert/delete operations.
- Existing Foundation text-replacement functionality is unchanged (backward compatible).
- Multi-operation batch (insert + delete + replace in same run): produces correct results via reverse-order processing.

## Phase 7: Structural Composition

### Scope

- Section operations: insert a section (heading + body blocks) as a unit; delete an entire section (heading + all body content until next same-level heading).
- Table creation: insert a new table with specified row/column dimensions and optional header row.
- List creation: insert a new bulletList or orderedList with specified items.
- Relaxation of `locked_structural` classification for block-level insert/delete operations (not attribute editing). Container nodes become insert/delete targets while preserving their wrapper structure.
- Template blocks: predefined structural patterns for common content shapes (section with heading + paragraph + table, FAQ pattern, etc.).

### Acceptance Criteria

- Insert a new section (H2 + paragraph + table) into an existing page: succeeds and publishes.
- Insert a new table with 3 rows x 2 columns: produces valid ADF and publishes.
- Delete an entire section (heading + all body content): succeeds and publishes.
- Insert a list with 5 items: produces valid ADF and publishes.
- Operations respect scope boundaries: no mutation outside `allowed_scope_paths`.
- Non-target content is completely untouched after compound structural operations.
- Block type conversion is explicitly out of scope (no paragraph-to-heading conversion).

## Phase 8: Multi-Page Content Control

### Scope

- Content-bearing sub-page creation: create new child pages with specified ADF structure (not just blank pages).
- Multi-page edit plans: define a set of coordinated operations across a parent page and its sub-pages with dependency ordering.
- Rollback checkpoints: on partial failure in a multi-page operation, roll back completed pages to their pre-operation state.
- Page hierarchy awareness: scope resolution understands parent/child relationships for cross-page references.
- Batch execution with page-level isolation: each page operation is independently verifiable before the batch commits.

### Acceptance Criteria

- Create a sub-page with specified heading/paragraph/table structure: succeeds and publishes.
- Edit content across parent + child pages in a single coordinated operation: all pages updated atomically.
- Failure on one page in a multi-page operation: other pages are rolled back to pre-operation state.
- Provenance tracking spans multi-page operations with per-page and batch-level metadata.
- Multi-page edit plan rejects cycles or unresolvable dependencies.

## Phase 9: Advanced Operations

### Scope

- Table topology changes: row add/remove, column add/remove within existing tables with strict bounds checking. Promoted from `ideas/2026-03-advanced-table-editing-modes.md`.
- Structural block attribute editing: metadata-safe updates for selected media and panel attributes, constrained macro/extension parameter edits. Promoted from `ideas/2026-03-structural-block-editing-support.md`.
- MCP server integration: expose the full insert/edit/delete/multi-page pipeline as MCP tools for AI agents. Promoted from `ideas/2026-03-mcp-server-integration.md`.

### Acceptance Criteria

- Add a row to an existing table: produces valid ADF and publishes.
- Remove a column from an existing table: produces valid ADF and publishes.
- Edit a panel's attributes without affecting its content: succeeds and publishes.
- MCP server exposes all pipeline operations with the same safety guarantees as the CLI.
- All Foundation and Structural safety gates (verification, scope enforcement, schema validation) apply equally to advanced operations.

## Dependencies and Planning Tracks

- Problem framing: `01-problem-points.md`
- KPI protocol: `04-kpi-and-experiments.md`
- Risk controls: `05-risks-and-mitigations.md`
- Execution checklist: `07-execution-readiness.md`
- Testing strategy and simulation: `10-testing-strategy-and-simulation.md`
- Lifecycle release-enablement track: `12-page-lifecycle-expansion-plan.md`

## OpenSpec Change Map

### Foundation (complete)

- `phase1-core-pipeline-skeleton-rust`
- `phase2-prose-assist-route-rust`
- `phase3-table-cell-route-rust`
- `phase4-poc-execution-metrics-rust`
- `phase5-hardening-readiness-rust`

### Structural (planned)

- `phase5.5-structural-refactor`
- `phase6-block-operation-foundation`
- `phase7-structural-composition`
- `phase8-multi-page-content-control`
- `phase9-advanced-operations`

These change IDs are planned execution containers under OpenSpec and should be used to track proposal/design/tasks and implementation progress.

## Explicitly Deferred

- Block type conversion (e.g., paragraph to heading, list to paragraphs).
- Inline node editing (mentions, status, emoji, date).
- Multi-space orchestration (cross-space page operations).
- Fully autonomous conflict resolution without human review.
- Stable-ID-based node addressing (upgrade from reverse-order processing if limitations are hit).

Previously deferred items now scheduled:

- Table row/column add/remove: Phase 9 (from `ideas/2026-03-advanced-table-editing-modes.md`).
- Structural block attribute editing: Phase 9 (from `ideas/2026-03-structural-block-editing-support.md`).
- Multi-page orchestration: Phase 8 (from `ideas/2026-03-multi-page-orchestration-and-autonomous-conflict-resolution.md`).
- MCP server integration: Phase 9 (from `ideas/2026-03-mcp-server-integration.md`).
