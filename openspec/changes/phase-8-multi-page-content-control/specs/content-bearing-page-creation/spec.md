## ADDED Requirements

### Requirement: Pages can be created with content in one coordinated step
When a `PageTarget` has `create: Some(CreatePageTarget)`, the `MultiPageOrchestrator` MUST first create the page, then run the per-page pipeline with the new page_id, `bootstrap_empty_page: true`, and the target's `block_ops`.

#### Scenario: Create sub-page with section content
- **GIVEN** a `PageTarget` with `create: Some(CreatePageTarget { title: "Report", parent_page_id: "123", space_key: "PROJ" })` and `block_ops: [InsertSection { ... }]`
- **WHEN** the target is processed
- **THEN** the orchestrator MUST call `create_page("Report", "123", "PROJ")` first
- **AND** THEN run the pipeline with the new page_id and `bootstrap_empty_page: true`
- **AND** the resulting page MUST contain the section content

#### Scenario: Page creation failure
- **WHEN** `create_page` fails (e.g., permission denied, network error)
- **THEN** the orchestrator MUST report `ERR_PAGE_CREATION_FAILED`
- **AND** MUST NOT run the pipeline for that page
- **AND** MUST proceed to rollback if `rollback_on_failure` is true

#### Scenario: Created page gets assigned page_id
- **WHEN** `create_page` returns `CreatePageResponse { page_id: "new-456" }`
- **THEN** the `RunRequest` MUST use `page_id: "new-456"`
- **AND** `PageResult.page_id` MUST be `"new-456"`
- **AND** `PageResult.created` MUST be `true`

### Requirement: CreatePageTarget carries creation config
The `CreatePageTarget` struct MUST contain `title: String`, `parent_page_id: String`, and `space_key: String`.

#### Scenario: CreatePageTarget round-trip
- **WHEN** a `CreatePageTarget` is serialized and deserialized
- **THEN** all fields MUST be preserved
