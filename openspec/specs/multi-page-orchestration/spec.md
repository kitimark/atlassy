## Purpose

Define multi-page orchestration behavior for coordinating per-page pipeline runs through a single execution plan.

## Requirements

### Requirement: MultiPageOrchestrator coordinates N page pipelines
The system SHALL provide a `MultiPageOrchestrator` that takes a `MultiPageRequest` and executes per-page pipelines in dependency order, returning a `MultiPageSummary`.

#### Scenario: All pages succeed
- **WHEN** a `MultiPageRequest` with 3 pages (no dependencies) is executed
- **THEN** all 3 pages MUST be processed sequentially via `Orchestrator::run()`
- **AND** `MultiPageSummary.success` MUST be `true`
- **AND** `page_results` MUST contain 3 entries, all with successful summaries

#### Scenario: One page fails, others not yet executed
- **GIVEN** a `MultiPageRequest` with pages A, B, C in order
- **WHEN** page B fails
- **THEN** page C MUST NOT be executed
- **AND** `MultiPageSummary.success` MUST be `false`
- **AND** `page_results` MUST contain A (success), B (failed)

#### Scenario: MultiPageOrchestrator wraps existing Orchestrator
- **WHEN** `MultiPageOrchestrator` processes a page
- **THEN** it MUST call `Orchestrator::run()` with a `RunRequest` built from the `PageTarget`
- **AND** the per-page pipeline (all 10 states) MUST execute unchanged

### Requirement: MultiPageRequest defines the execution plan
The `MultiPageRequest` MUST contain `plan_id`, `pages: Vec<PageTarget>`, `rollback_on_failure: bool`, `provenance`, and `timestamp`.

#### Scenario: Valid multi-page request
- **WHEN** a `MultiPageRequest` is constructed with valid pages and no dependency cycles
- **THEN** `MultiPageOrchestrator.run()` MUST accept and process it

#### Scenario: Empty pages list
- **WHEN** a `MultiPageRequest` has an empty `pages` vector
- **THEN** `MultiPageOrchestrator` MUST return success with empty results (no-op)

### Requirement: PageTarget carries per-page operation config
Each `PageTarget` MUST carry `page_id` (optional if creating), `create` (optional), `edit_intent`, `scope_selectors`, `run_mode`, `block_ops`, `bootstrap_empty_page`, and `depends_on`.

#### Scenario: PageTarget for existing page
- **WHEN** a `PageTarget` has `page_id: Some("12345")` and no `create` config
- **THEN** the orchestrator MUST use that page_id for the pipeline run

#### Scenario: PageTarget for new page
- **WHEN** a `PageTarget` has `page_id: None` and `create: Some(CreatePageTarget)`
- **THEN** the orchestrator MUST create the page first, then run the pipeline on the new page_id
