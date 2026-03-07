## Purpose

Define empty-page detection and bootstrap scaffold injection behavior for first-edit on blank pages, with explicit operator intent and deterministic safety guarantees.

## Requirements

### Requirement: Empty-page detection uses content-level analysis
The system SHALL determine whether a fetched page is effectively empty by analyzing the ADF content array. A page is effectively empty when its top-level `content` array is absent, empty, or contains only paragraphs whose own content is absent, empty, or contains only empty-string text nodes.

#### Scenario: Page with empty content array is detected as empty
- **WHEN** a fetched page has ADF `{"type": "doc", "version": 1, "content": []}`
- **THEN** the page is classified as effectively empty

#### Scenario: Page with single empty paragraph is detected as empty
- **WHEN** a fetched page has ADF containing a single paragraph with no content or only empty-string text
- **THEN** the page is classified as effectively empty

#### Scenario: Page with substantive content is detected as non-empty
- **WHEN** a fetched page has ADF containing headings, paragraphs with non-empty text, tables, or other content nodes
- **THEN** the page is classified as non-empty

#### Scenario: Page with only structural nodes and no text is detected as non-empty
- **WHEN** a fetched page has ADF containing non-paragraph nodes (panels, macros, tables) even without visible text
- **THEN** the page is classified as non-empty

### Requirement: Bootstrap requires explicit operator intent
The `--bootstrap-empty-page` flag MUST be explicitly provided by the operator. The system SHALL NOT implicitly bootstrap empty pages.

#### Scenario: Empty page without bootstrap flag hard-fails
- **WHEN** a pipeline run fetches a page that is effectively empty
- **AND** the `--bootstrap-empty-page` flag is not set
- **THEN** the run fails with error code `ERR_BOOTSTRAP_REQUIRED`
- **AND** `failure_state` is set to `Fetch`

#### Scenario: Empty page with bootstrap flag proceeds
- **WHEN** a pipeline run fetches a page that is effectively empty
- **AND** the `--bootstrap-empty-page` flag is set
- **THEN** the run injects a minimal prose scaffold into the fetched ADF
- **AND** the pipeline continues through classify and subsequent states

### Requirement: Bootstrap on non-empty page is rejected
The system SHALL hard-fail when the `--bootstrap-empty-page` flag is set but the fetched page already has content.

#### Scenario: Non-empty page with bootstrap flag hard-fails
- **WHEN** a pipeline run fetches a page that is not effectively empty
- **AND** the `--bootstrap-empty-page` flag is set
- **THEN** the run fails with error code `ERR_BOOTSTRAP_INVALID_STATE`
- **AND** `failure_state` is set to `Fetch`

### Requirement: Non-empty page without flag follows unchanged flow
The system SHALL NOT alter pipeline behavior for non-empty pages when the bootstrap flag is not set.

#### Scenario: Non-empty page without bootstrap flag is unchanged
- **WHEN** a pipeline run fetches a page that is not effectively empty
- **AND** the `--bootstrap-empty-page` flag is not set
- **THEN** the pipeline proceeds through all states with unchanged v1 behavior

### Requirement: Bootstrap scaffold contains minimal prose-only ADF
When bootstrap is triggered, the system SHALL replace the empty ADF with a minimal scaffold containing one heading and one paragraph, both with empty text content. The scaffold MUST contain only `editable_prose` route nodes.

#### Scenario: Scaffold contains heading and paragraph
- **WHEN** bootstrap is triggered on an empty page
- **THEN** the injected ADF contains a level-2 heading with empty text and a paragraph with empty text
- **AND** no `table_adf` or `locked_structural` nodes are present in the scaffold

### Requirement: Bootstrap detection occurs after fetch and before classify
Bootstrap evaluation SHALL occur after the fetch state completes and before the classify state begins. The 9-state pipeline ordering and `PipelineState` enum MUST NOT be modified.

#### Scenario: Bootstrap modifies fetch output before classify sees it
- **WHEN** bootstrap injects a scaffold
- **THEN** the classify state receives the scaffolded ADF, not the original empty ADF
- **AND** no additional pipeline state is added to the state tracker

### Requirement: Bootstrap markers appear in run summary telemetry
Run summaries for bootstrap-involved runs SHALL include markers indicating whether the page was detected as empty and whether bootstrap was applied.

#### Scenario: Summary records bootstrap application
- **WHEN** a bootstrap run completes (success or failure)
- **THEN** the run summary includes `empty_page_detected` and `bootstrap_applied` boolean fields

#### Scenario: Non-bootstrap runs have false markers
- **WHEN** a non-bootstrap run completes on a non-empty page
- **THEN** the run summary includes `empty_page_detected: false` and `bootstrap_applied: false`
