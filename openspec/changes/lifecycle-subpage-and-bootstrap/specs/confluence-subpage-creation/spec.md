## ADDED Requirements

### Requirement: Sub-page creation uses explicit parent targeting and space key
The system SHALL create child pages under an explicitly provided parent page ID within an explicitly provided space key. The create operation MUST NOT infer the space from the parent page.

#### Scenario: Create blank sub-page with valid parent and space
- **WHEN** `create-subpage` is invoked with a valid `parent_page_id`, `space_key`, and `title`
- **THEN** a new child page is created under the specified parent in the specified space
- **AND** the response includes the created `page_id` and `page_version`

#### Scenario: Create sub-page with missing parent
- **WHEN** `create-subpage` is invoked with a `parent_page_id` that does not exist
- **THEN** the operation fails with a deterministic not-found error
- **AND** operator-facing output identifies the missing parent page

#### Scenario: Create sub-page with invalid space key
- **WHEN** `create-subpage` is invoked with a `space_key` that does not exist
- **THEN** the operation fails with a deterministic not-found error

### Requirement: Created pages are blank by default
The system SHALL create pages with an empty ADF document body. No automatic seed content, template content, or default text SHALL be inserted at creation time.

#### Scenario: Blank page body on creation
- **WHEN** a sub-page is created via `create-subpage`
- **THEN** the page body contains an empty ADF document (`{"type": "doc", "version": 1, "content": []}`)
- **AND** the page version is `1`

### Requirement: Duplicate title creation fails deterministically
The system SHALL fail deterministically when creating a page with a title that already exists in the target space under the same parent.

#### Scenario: Duplicate title rejection
- **WHEN** `create-subpage` is invoked with a title that already exists in the target space
- **THEN** the operation fails with a deterministic error
- **AND** operator-facing output indicates the title collision

### Requirement: Sub-page creation does not involve the pipeline orchestrator
The `create-subpage` command SHALL call the Confluence client directly without instantiating an orchestrator or running any pipeline states.

#### Scenario: No pipeline states execute during creation
- **WHEN** `create-subpage` is invoked
- **THEN** no pipeline state artifacts are persisted
- **AND** no state tracker transitions occur

### Requirement: Sub-page creation works in both stub and live backends
The `create_page` client method SHALL be implemented for both `stub` and `live` runtime backends with deterministic behavior.

#### Scenario: Stub backend creation
- **WHEN** `create-subpage` is invoked with the `stub` runtime backend
- **THEN** the page is created in the in-memory stub page store
- **AND** the returned `page_id` is deterministic given the same inputs

#### Scenario: Live backend creation
- **WHEN** `create-subpage` is invoked with the `live` runtime backend and valid credentials
- **THEN** the page is created via the Confluence REST API
- **AND** the response includes the Confluence-assigned `page_id` and version `1`

#### Scenario: Live backend creation with auth failure
- **WHEN** `create-subpage` is invoked with the `live` runtime backend and invalid or missing credentials
- **THEN** the operation fails with a deterministic runtime backend error
