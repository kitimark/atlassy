## Purpose

Define CLI behavior for executing multi-page manifests through the multi-page orchestration pipeline.

## Requirements

### Requirement: CLI provides run-multi-page command
The CLI SHALL provide a `run-multi-page` command that reads a `MultiPageRequest` manifest from a JSON file and executes it via `MultiPageOrchestrator`.

#### Scenario: Successful multi-page run
- **WHEN** `atlassy run-multi-page --manifest plan.json --artifacts-dir ./out --runtime-backend stub` is executed
- **THEN** the CLI MUST parse `plan.json` as `MultiPageRequest`
- **AND** execute via `MultiPageOrchestrator`
- **AND** print `MultiPageSummary` as JSON to stdout

#### Scenario: Invalid manifest file
- **WHEN** the manifest file is not valid JSON or doesn't match `MultiPageRequest` schema
- **THEN** the CLI MUST exit with a non-zero status and a clear error message

#### Scenario: Manifest not found
- **WHEN** the manifest file path does not exist
- **THEN** the CLI MUST exit with a non-zero status

### Requirement: CLI run-multi-page accepts runtime backend selection
The `run-multi-page` command MUST accept `--runtime-backend` with values `stub` or `live`, matching the existing `run` and `run-batch` commands.

#### Scenario: Stub backend
- **WHEN** `--runtime-backend stub` is specified
- **THEN** the CLI MUST use `StubConfluenceClient`

#### Scenario: Live backend
- **WHEN** `--runtime-backend live` is specified
- **THEN** the CLI MUST use `LiveConfluenceClient`

### Requirement: MultiPageRequest manifest format
The manifest MUST be a JSON file deserializable as `MultiPageRequest` with `plan_id`, `pages`, `rollback_on_failure`, `provenance`, and `timestamp` fields.

#### Scenario: Manifest with mixed existing and new pages
- **WHEN** the manifest contains pages with `page_id` set and pages with `create` set
- **THEN** the CLI MUST accept the manifest and process both types
