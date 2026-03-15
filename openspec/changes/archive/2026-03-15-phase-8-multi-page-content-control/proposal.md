## Why

The pipeline operates on a single page per run. Users need to coordinate edits across parent and child pages — updating a summary on a parent page while creating or modifying detail pages underneath — with rollback if any page fails. Phase 8 adds a multi-page orchestration layer that wraps the existing per-page pipeline without modifying it, following the Facade and Mediator patterns.

## What Changes

- New `MultiPageOrchestrator` in `atlassy-pipeline` that coordinates N single-page pipeline runs with dependency ordering, snapshot-based rollback, and content-bearing page creation (Facade + Mediator patterns).
- New types in `atlassy-contracts`: `MultiPageRequest`, `PageTarget`, `CreatePageTarget`, `PageSnapshot`, `MultiPageSummary`, `PageResult`, `RollbackResult`.
- Versioned rollback using the Memento pattern: save page state (ADF + version) before modification, restore on failure with conflict detection (don't overwrite concurrent edits).
- Content-bearing sub-page creation: `PageTarget.create` config triggers `create_page` + pipeline run with `bootstrap_empty_page` and `block_ops` in one coordinated step (Single Responsibility: creation separate from content operations).
- Dependency ordering via topological sort on `PageTarget.depends_on` — cycle detection rejects invalid plans.
- New CLI command `run-multi-page` that reads a multi-page manifest and executes coordinated operations.
- 4 new error codes: `MultiPagePartialFailure`, `RollbackConflict`, `DependencyCycle`, `PageCreationFailed`.
- Existing single-page `Orchestrator.run()` is completely unchanged — all 10 pipeline states are untouched. ADF crate is unchanged. Confluence trait is unchanged.

## Capabilities

### New Capabilities

- `multi-page-orchestration`: The `MultiPageOrchestrator` that coordinates N single-page pipelines — validates plan, takes snapshots, executes in dependency order, handles failures, performs rollback. Facade over the existing per-page Orchestrator.
- `page-snapshot-rollback`: Memento-based page state snapshots (`PageSnapshot`) with versioned rollback on multi-page failure. Conflict detection prevents overwriting concurrent edits during rollback.
- `dependency-ordering`: Topological sort on `PageTarget.depends_on` for execution ordering. Cycle detection rejects invalid dependency graphs before execution.
- `content-bearing-page-creation`: Create new sub-pages with content in one coordinated step — `create_page` followed by pipeline run with `bootstrap_empty_page` + `block_ops`.
- `multi-page-cli-command`: CLI `run-multi-page` command that reads a `MultiPageRequest` manifest and executes coordinated multi-page operations.

### Modified Capabilities

- `typed-error-codes`: 4 new error codes for multi-page failures (MultiPagePartialFailure, RollbackConflict, DependencyCycle, PageCreationFailed).
- `pipeline-state-orchestration`: Pipeline crate exports `MultiPageOrchestrator` alongside existing `Orchestrator`. No changes to per-page orchestration or state execution.

## Impact

- **atlassy-contracts**: 7 new types, 4 new error codes.
- **atlassy-pipeline**: New `multi_page.rs` module with `MultiPageOrchestrator`. Existing `orchestrator.rs` and all states unchanged.
- **atlassy-cli**: New `run-multi-page` command and manifest parsing.
- **atlassy-adf**: No changes.
- **atlassy-confluence**: No trait changes. Existing `create_page`, `fetch_page`, `publish_page` methods used for creation, snapshots, and rollback.
