## MODIFIED Requirements

### Requirement: Pipeline crate exports MultiPageOrchestrator
The `atlassy-pipeline` crate MUST export `MultiPageOrchestrator` alongside the existing `Orchestrator`. Both are public types available to CLI and other consumers.

#### Scenario: MultiPageOrchestrator is constructable
- **WHEN** a `MultiPageOrchestrator` is created with a `ConfluenceClient` and artifact root
- **THEN** it MUST be usable to run `MultiPageRequest` plans

#### Scenario: Existing Orchestrator unchanged
- **WHEN** `Orchestrator::run()` is called with a single `RunRequest`
- **THEN** behavior MUST be identical to pre-Phase-8 (no changes to per-page pipeline)

#### Scenario: MultiPageOrchestrator uses Orchestrator internally
- **WHEN** `MultiPageOrchestrator` processes a page
- **THEN** it MUST delegate to `Orchestrator::run()` for per-page pipeline execution
- **AND** it MUST NOT duplicate or reimplement per-page pipeline logic
