## Purpose

Define snapshot capture and rollback behavior for multi-page runs that partially succeed before a failure.

## Requirements

### Requirement: Page state is saved before modification for rollback
The `MultiPageOrchestrator` MUST save a `PageSnapshot` for each existing page before running its pipeline. The snapshot contains `page_id`, `version_before`, and `adf_before`.

#### Scenario: Snapshot taken before pipeline execution
- **WHEN** the orchestrator is about to process an existing page
- **THEN** it MUST call `fetch_page` and save the current ADF and version as a `PageSnapshot`
- **AND** the snapshot MUST be saved before any modification occurs

#### Scenario: Snapshot not taken for newly created pages
- **WHEN** a `PageTarget` creates a new page (no prior state exists)
- **THEN** no pre-modification snapshot is saved for that page

### Requirement: Versioned rollback restores page state on failure
When `rollback_on_failure` is true and a page fails, all successfully published pages MUST be rolled back by re-publishing their original ADF, with version conflict detection.

#### Scenario: Successful rollback
- **GIVEN** page A was published successfully (version 4 -> 5)
- **WHEN** page B fails and rollback is triggered
- **THEN** the orchestrator MUST fetch page A's current version
- **AND** if current version == 5 (our publish): re-publish original ADF -> `RollbackResult { success: true }`

#### Scenario: Rollback conflict detected
- **GIVEN** page A was published (version 4 -> 5), then someone else edited it (version -> 6)
- **WHEN** rollback is triggered
- **THEN** the orchestrator MUST detect version mismatch (current 6 != expected 5)
- **AND** return `RollbackResult { success: false, conflict: true }`
- **AND** MUST NOT overwrite the concurrent edit

#### Scenario: Rollback processes pages in reverse order
- **GIVEN** pages A, B, C were published in order, D failed
- **WHEN** rollback is triggered
- **THEN** rollback MUST process C, B, A (reverse publication order)

#### Scenario: Rollback skips created pages
- **GIVEN** page X was created during the multi-page operation
- **WHEN** rollback is triggered
- **THEN** page X MUST NOT be deleted or modified during rollback

#### Scenario: No rollback when disabled
- **WHEN** `rollback_on_failure` is false and a page fails
- **THEN** no rollback MUST be attempted
- **AND** `MultiPageSummary.rollback_results` MUST be empty
